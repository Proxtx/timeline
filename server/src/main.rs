mod api;
mod config;
mod plugin_registry;
mod proxy;

use std::io;
use std::path::PathBuf;

use rocket::fs::{FileServer, NamedFile, Options};
use rocket::response::{content, status};
use rocket::{catch, catchers, routes, Request};

use crate::config::Config;
use crate::plugin_registry::PluginRegistry;

#[rocket::launch]
async fn rocket() -> _ {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::load("config.toml")
        .await
        .unwrap_or_else(|e| panic!("unable to load config.toml: {}", e));

    tokio::fs::create_dir_all(&config.data_dir)
        .await
        .unwrap_or_else(|e| panic!("unable to create data_dir: {}", e));
    tokio::fs::create_dir_all(config.data_dir.join("plugin_web"))
        .await
        .ok();

    let registry = PluginRegistry::new(&config.plugin);
    tracing::info!(count = config.plugin.len(), "plugins registered");

    let plugin_web_root = config.data_dir.join("plugin_web");
    let figment = rocket::Config::figment().merge(("port", config.port));

    rocket::custom(figment)
        .register("/", catchers![not_found])
        .manage(config)
        .manage(registry)
        .mount("/", FileServer::from("../frontend/dist/").rank(20))
        .mount(
            "/plugin_web",
            FileServer::new(plugin_web_root, Options::Index | Options::DotFiles).rank(5),
        )
        .mount(
            "/api",
            routes![
                api::auth_request,
                api::events,
                api::markers,
                api::plugins,
                proxy::proxy_get,
                proxy::proxy_post,
                proxy::proxy_put,
                proxy::proxy_delete,
            ],
        )
}

#[catch(404)]
async fn not_found(
    _req: &Request<'_>,
) -> Result<status::Accepted<content::RawHtml<NamedFile>>, io::Error> {
    let path = PathBuf::from("../frontend/dist/index.html");
    match NamedFile::open(path).await {
        Ok(f) => Ok(status::Accepted(content::RawHtml(f))),
        Err(e) => Err(e),
    }
}
