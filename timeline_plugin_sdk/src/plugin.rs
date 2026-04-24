//! The [`Plugin`] trait that every plugin implements.

use std::time::Duration;

use rocket::Route;

use types::api::{APIResult, CompressedEvent};
use types::timing::TimeRange;

use crate::assets::AssetStore;
use crate::cache::Cache;
use crate::config::PluginConfig;
use crate::db::Db;
use crate::error::ErrorReporter;
use crate::manifest::Manifest;

/// Passed to [`Plugin::new`] once at startup. Gives the plugin everything the
/// SDK set up on its behalf.
pub struct Context {
    pub config: PluginConfig,
    /// Plugin-specific sub-config extracted from the `[config]` table.
    pub extra: toml::Value,
    pub db: Db,
    pub assets: AssetStore,
    pub cache: Cache,
    pub errors: ErrorReporter,
}

/// A timeline plugin. Implementations run in their own process.
///
/// `events` is mandatory. Everything else is optional and defaults to a no-op.
pub trait Plugin: Sized + Send + Sync + 'static {
    /// Constructor. Called once at startup.
    fn new(ctx: Context) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send;

    /// Return the plugin manifest (cheap; called on every `GET /manifest`).
    fn manifest(&self) -> Manifest;

    /// Return the events whose timing overlaps `range`.
    fn events(
        &self,
        range: TimeRange,
    ) -> impl std::future::Future<Output = APIResult<Vec<CompressedEvent>>> + Send;

    /// Optional background loop. The SDK re-runs this after each returned
    /// `Duration`; return `None` to stop. Panics are caught by the SDK and
    /// reported via [`ErrorReporter`].
    fn request_loop(&self) -> impl std::future::Future<Output = Option<Duration>> + Send {
        async { None }
    }

    /// Plugin-specific Rocket routes. Mounted by the SDK at the same origin
    /// as the standard endpoints — i.e. the main server proxies them through
    /// `/api/plugin/<name>/<your-path>`. Returning an empty vec is fine.
    fn routes(&self) -> Vec<Route> {
        Vec::new()
    }
}
