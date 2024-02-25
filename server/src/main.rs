use cache::Cache;
use db::Database;

mod cache;
mod config;
mod db;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
/*#[path = "../plugins/test.rs"]
mod test;*/
//TODO: fix the import in the build script to import from absolute path

pub trait Plugin {
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

    let t = Plugins::init(|_plugin_name| PluginData { database: &db }).await;
}

pub struct PluginData<'a> {
    pub database: &'a Database,
}
