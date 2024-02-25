use db::Database;

mod config;
mod db;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
#[path = "../plugins/test.rs"]
mod test;

pub trait Plugin {
    fn init(&self);
    fn get_type(&self) -> AvailablePlugins;
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
    t.plugins["test"].init();
}

pub struct PluginData<'a> {
    pub database: &'a Database,
}
