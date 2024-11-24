use {
    crate::plugin_manager::PluginManager,
    rocket::{
        fs::NamedFile,
        get,
        http::{CookieJar, Status},
        post,
        response::status,
        serde::json::Json,
        State,
    },
    serde::Deserialize,
    server_api::{
        config::Config,
        db::Database,
        external::types::{
            api::{ APIResult, CompressedEvent},
            available_plugins::AvailablePlugins,
            external::{
                chrono::{DateTime, SubsecRound, Timelike, Utc},
                mongodb::{bson::doc, options::FindOptions},
            },
            timing::{Marker, TimeRange, Timing},
        },
        web::auth,
    },
    std::{collections::HashMap, path::PathBuf, sync::Arc},
};

pub mod markers {
    use super::*;

    pub async fn get_markers(range: &TimeRange, database: &Database) -> APIResult<Vec<Marker>> {
        #[derive(Deserialize)]
        struct OnlyTimingEvent {
            timing: Timing,
        }

        let mut events = database
            .find_events_with_custom_query::<OnlyTimingEvent>(
                Database::generate_range_filter(range),
                FindOptions::builder()
                    .projection(doc! {"timing": 1})
                    .build(),
            )
            .await?;

        let mut hour_events: HashMap<DateTime<Utc>, u32> = HashMap::new();

        while events.advance().await? {
            let next_event = events.deserialize_current()?;
            let time = match next_event.timing {
                Timing::Instant(t) => t,
                Timing::Range(range) => range.start,
            };

            let new_time = time
                .round_subsecs(0)
                .with_second(0)
                .unwrap()
                .with_minute(0)
                .unwrap();
            match hour_events.get_mut(&new_time) {
                Some(v) => {
                    *v += 1;
                }
                None => {
                    hour_events.insert(new_time, 1);
                }
            }
        }

        let mut res: Vec<_> = hour_events
            .into_iter()
            .map(|(time, amount)| Marker { time, amount })
            .collect();

        res.sort_by(|a, b| b.amount.cmp(&a.amount));
        res = res.into_iter().collect();

        Ok(res)
    }
    #[post("/markers", data = "<request>")]
    pub async fn get_markers_request(
        request: Json<TimeRange>,
        config: &State<Config>,
        database: &State<Arc<Database>>,
        cookies: &CookieJar<'_>,
    ) -> status::Custom<Json<APIResult<Vec<Marker>>>> {
        if let Err(e) = auth(cookies, config) {
            status::Custom(Status::Unauthorized, Json(Err(e)))
        } else {
            status::Custom(Status::Ok, Json(get_markers(&request, database).await))
        }
    }
}

pub mod events {
    use super::*;

    #[post("/events", data = "<request>")]
    pub async fn get_events(
        request: Json<TimeRange>,
        config: &State<Config>,
        plugin_manager: &State<PluginManager>,
        cookies: &CookieJar<'_>,
    ) -> status::Custom<Json<APIResult<HashMap<AvailablePlugins, Vec<CompressedEvent>>>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }
        match plugin_manager.get_compress_events(&request).await {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => status::Custom(Status::InternalServerError, Json(Err(e))),
        }
    }

    #[get("/icon/<plugin>")]
    pub async fn get_icon(plugin: &str) -> Option<NamedFile> {
        let mut path = PathBuf::from("../plugins/");
        path.push(plugin);
        path.push("icon.svg");
        NamedFile::open(path).await.ok()
    }
}

#[cfg(feature = "experiences")]
#[post("/experiences_url")]
pub fn experiences_url(config: &State<Config>) -> status::Accepted<Json<APIResult<String>>> {
    status::Accepted(Json(Ok(config.experiences_url.to_string())))
}

#[post("/auth")]
pub fn auth_request(
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> status::Custom<Json<APIResult<()>>> {
    status::Custom(Status::Ok, Json(auth(cookies, config)))
}
