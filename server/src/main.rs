#![feature(unboxed_closures)]
#![feature(fn_traits)]

mod api;
mod plugin_manager;

use {
    dyn_link::server_plugins::Plugins,
    rocket::{
        catch, catchers,
        fs::FileServer,
        response::{content, status},
        routes, Request,
    },
    server_api::{
        config, db,
        external::{tokio::fs::File, types::api::CompressedEvent},
        plugin::PluginData,
    },
    std::{io, sync::Arc},
};

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

    let plugins = Plugins::init(|plugin| PluginData {
        database: db.clone(),
        config: config.plugin_config.remove(&plugin),
        plugin,
        error_url: config.error_report_url.clone(),
    })
    .await;

    let plugin_manager = plugin_manager::PluginManager::new(plugins.plugins);

    let figment = rocket::Config::figment().merge(("port", config.port));
    let mut rocket_state = rocket::custom(figment)
        .register("/", catchers![not_found])
        .manage(config)
        .manage(db)
        .mount("/", FileServer::from("../frontend/dist/"))
        .mount(
            "/api",
            routes![
                api::markers::get_markers_request,
                api::events::get_events,
                api::events::get_icon,
                api::auth_request,
            ],
        );

    #[cfg(feature = "experiences")]
    {
        rocket_state = rocket_state.mount("/api", routes![api::experiences_url]);
    }

    for (plugin, routes) in plugins.routes {
        rocket_state = rocket_state.mount(format!("/api/plugin/{}", plugin), routes);
        rocket_state = plugin_manager
            .get_plugin(&plugin)
            .read()
            .await
            .rocket_build_access(rocket_state);
    }

    rocket_state = rocket_state.manage(plugin_manager);
    rocket_state
}

#[catch(404)]
async fn not_found(
    _req: &Request<'_>,
) -> Result<status::Accepted<content::RawHtml<File>>, io::Error> {
    match File::open("../frontend/dist/index.html").await {
        Ok(v) => Ok(status::Accepted(content::RawHtml(v))),
        Err(e) => Err(e),
    }
}
