#![feature(unboxed_closures)]
#![feature(fn_traits)]

use chrono::Duration;
use db::Database;
use std::pin::Pin;

mod cache;
mod config;
mod db;
mod plugin_manager;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
#[path = "../plugins/timeline_plugin_media_scan/plugin.rs"]
mod test;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin<'a>
where
    Self: Send,
{
    fn new(data: PluginData<'a>) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_type() -> AvailablePlugins
    where
        Self: Sized;

    fn request_loop(&mut self) -> Pin<Box<dyn futures::Future<Output = Option<Duration>> + Send>>;
}

#[tokio::main]
async fn main() {
    let config = config::Config::load()
        .await
        .unwrap_or_else(|e| panic!("Unable to init Config: {}", e));

    let db = db::Database::new(&config.db_connection_string, &config.database)
        .await
        .unwrap_or_else(|e| {
            panic! {"Unable to connect to Database: {}", e};
        });

    let t = Plugins::init(|plugin| PluginData {
        database: &db,
        config: config.plugin_config.get(&plugin),
    })
    .await;

    let plugin_manager = plugin_manager::PluginManager::new(t.plugins);
}

pub struct PluginData<'a> {
    pub database: &'a Database,
    pub config: Option<&'a toml::Value>,
}
