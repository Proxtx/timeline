use db::Database;

mod cache;
mod config;
mod db;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
/*#[path = "../plugins/test.rs"]
mod test;*/
#[path = "../plugins/timeline_plugin_media_scan/plugin.rs"]
mod timeline_plugin_media_scan;

pub trait Plugin<'a> {
    async fn new(data: PluginData<'a>) -> Self
    where
        Self: Sized;
    fn get_type() -> AvailablePlugins
    where
        Self: Sized;
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
}

pub struct PluginData<'a> {
    pub database: &'a Database,
    pub config: Option<&'a toml::Value>,
}
