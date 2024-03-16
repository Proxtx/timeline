#![feature(unboxed_closures)]
#![feature(fn_traits)]

use chrono::Duration;
use db::Database;
use rocket::fs::FileServer;
use rocket::response::content;
use rocket::response::status;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Barrier;
use rocket::catch;
use rocket::catchers;
use rocket::Request;
use tokio::fs::File;
use rocket::response::stream::ReaderStream;

mod api;
mod cache;
mod config;
mod db;
mod plugin_manager;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));

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
    .manage(config)
    .manage(db)
    .mount("/", FileServer::from("../frontend/dist/"))
}

#[catch(404)]
async fn not_found(_req: &Request<'_>) -> Result<status::Accepted<content::RawHtml<File>>, io::Error> {
    match File::open("../frontend/dist/index.html").await {
        Ok(v) => Ok(status::Accepted(content::RawHtml(v))),
        Err(e) => Err(e)
    }
}

pub struct PluginData {
    pub database: Arc<Database>,
    pub config: Option<toml::Value>,
}
