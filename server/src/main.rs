#![feature(unboxed_closures)]
#![feature(fn_traits)]

use chrono::Duration;
use db::Database;
use rocket::fs::FileServer;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Barrier;
use rocket::catch;
use rocket::catchers;
use rocket::Request;
use tokio::fs::File;
use rocket::response::stream::ReaderStream;

mod cache;
mod config;
mod db;
mod plugin_manager;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
#[allow(clippy::duplicate_mod)]
#[path = "../../plugins/timeline_plugin_media_scan/plugin.rs"]
mod _i1;

pub trait Plugin: Send + Sync {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_type() -> AvailablePlugins
    where
        Self: Sized;

    fn request_loop_mut<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn futures::Future<Output = Option<Duration>> + Send + 'a>> {
        Box::pin(async move {None})
    }

    fn request_loop<'a> (
        &'a self
    ) -> Pin<Box<dyn futures::Future<Output = Option<Duration>> + Send + 'a>> {
        Box::pin(async move {None})
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    let mut config = config::Config::load()
        .await
        .unwrap_or_else(|e| panic!("Unable to init Config: {}", e));

    let db = Arc::new(
        db::Database::new(&config.db_connection_string, &config.database)
            .await
            .unwrap_or_else(|e| {
                panic! {"Unable to connect to Database: {}", e};
            }),
    );

    let plgs = Plugins::init(|plugin| PluginData {
        database: db.clone(),
        config: config.plugin_config.remove(&plugin),
    })
    .await;

    let plugin_manager = plugin_manager::PluginManager::new(plgs.plugins);

    let figment = rocket::Config::figment().merge(("port", config.port));
    rocket::custom(figment)
    .register("/", catchers![not_found])
    .manage(plugin_manager)
    .mount("/", FileServer::from("../frontend/dist/"))
}

#[catch(404)]
fn not_found(req: &Request) -> ReaderStream![File] {
    ReaderStream! {
        if let Ok(file) = File::open("../frontend/dist/index.html").await {
            yield file;
        }
    }
}

pub struct PluginData {
    pub database: Arc<Database>,
    pub config: Option<toml::Value>,
}
