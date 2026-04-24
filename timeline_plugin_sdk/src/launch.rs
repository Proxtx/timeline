//! Top-level `launch::<MyPlugin>()` glue.

use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use futures::FutureExt;
use rocket::{routes, Config as RocketConfig, Route};

use types::api::{APIResult, CompressedEvent};
use types::timing::TimeRange;

use crate::assets::AssetStore;
use crate::cache::Cache;
use crate::config::BaseConfig;
use crate::db::Db;
use crate::error::ErrorReporter;
use crate::manifest::Manifest;
use crate::plugin::{Context, Plugin};

/// Type-erased plugin handle behind a trait object. Rocket state holds one
/// of these so the standard routes can call into whichever concrete plugin
/// was launched.
#[derive(Clone)]
pub struct PluginHandle {
    inner: Arc<dyn PluginObj>,
}

impl PluginHandle {
    fn new<P: Plugin>(plugin: Arc<P>) -> Self {
        Self { inner: plugin }
    }

    pub fn manifest(&self) -> Manifest {
        self.inner.obj_manifest()
    }

    pub async fn events(&self, range: TimeRange) -> APIResult<Vec<CompressedEvent>> {
        self.inner.obj_events(range).await
    }
}

/// Ambient state every plugin route can look up:
///  - the bearer token (for the auth guard)
///  - asset store (for `/assets/<path..>`)
pub struct PluginState {
    pub token: String,
    pub plugin_name: String,
    pub assets: AssetStore,
    pub db: Db,
    pub cache: Cache,
    pub errors: ErrorReporter,
}

/// Entry point. A plugin's `main` is typically just:
///
/// ```ignore
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     timeline_plugin_sdk::launch::<MyPlugin>("config.toml").await
/// }
/// ```
pub async fn launch<P: Plugin>(config_path: impl Into<PathBuf>) -> anyhow::Result<()> {
    let cfg = BaseConfig::load(config_path.into()).await?;

    let plugin_root = cfg.plugin.plugin_root();
    tokio::fs::create_dir_all(&plugin_root).await?;

    let db = Db::open(cfg.plugin.db_path()).await?;
    let assets = AssetStore::open(cfg.plugin.assets_root()).await?;
    let cache = Cache::open(cfg.plugin.cache_root()).await?;
    let errors = ErrorReporter::new(cfg.plugin.name.clone(), cfg.plugin.error_report_url.clone());

    let ctx = Context {
        config: cfg.plugin.clone(),
        extra: cfg.config.clone(),
        db: db.clone(),
        assets: assets.clone(),
        cache: cache.clone(),
        errors: errors.clone(),
    };

    let plugin = Arc::new(P::new(ctx).await?);
    let handle = PluginHandle::new(plugin.clone());

    spawn_request_loop(plugin.clone(), cfg.plugin.name.clone(), errors.clone());

    let state = PluginState {
        token: cfg.plugin.token.clone(),
        plugin_name: cfg.plugin.name.clone(),
        assets,
        db,
        cache,
        errors,
    };

    let plugin_routes = plugin.obj_routes();

    let rocket_cfg = RocketConfig::figment().merge(("port", cfg.plugin.port));

    let mut rocket = rocket::custom(rocket_cfg)
        .manage(state)
        .manage(handle)
        .mount(
            "/",
            routes![
                crate::routes::events,
                crate::routes::manifest,
                crate::routes::health,
                crate::routes::assets,
            ],
        );

    if !plugin_routes.is_empty() {
        rocket = rocket.mount("/", plugin_routes);
    }

    rocket.launch().await?;
    Ok(())
}

fn spawn_request_loop<P: Plugin>(plugin: Arc<P>, name: String, errors: ErrorReporter) {
    tokio::spawn(async move {
        loop {
            let fut = plugin.obj_request_loop();
            let outcome = AssertUnwindSafe(fut).catch_unwind().await;
            match outcome {
                Ok(Some(d)) => tokio::time::sleep(d).await,
                Ok(None) => break,
                Err(panic) => {
                    let msg = panic_message(&panic);
                    errors.report(format!("{} request_loop panicked: {}", name, msg));
                    tokio::time::sleep(Duration::from_secs(300)).await;
                }
            }
        }
    });
}

fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

// --------- type-erased plugin object --------

trait PluginObj: Send + Sync + 'static {
    fn obj_manifest(&self) -> Manifest;
    fn obj_events<'a>(&'a self, range: TimeRange) -> BoxFuture<'a, APIResult<Vec<CompressedEvent>>>;
    fn obj_request_loop<'a>(&'a self) -> BoxFuture<'a, Option<Duration>>;
    fn obj_routes(&self) -> Vec<Route>;
}

impl<P: Plugin> PluginObj for P {
    fn obj_manifest(&self) -> Manifest {
        Plugin::manifest(self)
    }
    fn obj_events<'a>(&'a self, range: TimeRange) -> BoxFuture<'a, APIResult<Vec<CompressedEvent>>> {
        Box::pin(Plugin::events(self, range))
    }
    fn obj_request_loop<'a>(&'a self) -> BoxFuture<'a, Option<Duration>> {
        Box::pin(Plugin::request_loop(self))
    }
    fn obj_routes(&self) -> Vec<Route> {
        Plugin::routes(self)
    }
}
