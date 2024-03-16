use types::api::APIResult;
use types::api::APIError;
use crate::db::DatabaseError;
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};

pub mod markers {
    use crate::{config::Config, db::{Database, DatabaseError, Event}};
    use chrono::{DateTime, SubsecRound, Timelike, Utc};
    use futures::StreamExt;
    use mongodb::{bson::{doc, Document}, options::FindOptions};
    use rocket::{post, request::FromRequest, serde::json::Json, State};
    use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
    use std::{collections::HashMap, fmt::{self, format}};
    use std::sync::Arc;
    use rocket::response::status;
    use rocket::http::Status;
    use types::timing::{Marker, TimeRange, Timing};
    use types::api::APIError;
    use types::api::APIResult;
    use rocket::http::CookieJar;

    pub async fn get_markers(range: &TimeRange, database: &Database) -> APIResult<Vec<Marker>> {
        #[derive(Deserialize)]
        struct OnlyTimingEvent {
            timing: Timing
        }

        let mut events = database
            .find_events_with_custom_query::<OnlyTimingEvent>(Database::generate_range_filter(range), FindOptions::builder().sort(doc! {
                "timing.0": 1
            }).build()).await?;
        
        let mut hour_events: HashMap<DateTime<Utc>, u32> = HashMap::new();

        while events.advance().await? {
            let next_event = events.deserialize_current()?;
            let time = match next_event.timing {
                Timing::Instant(t) => {
                    t
                }
                Timing::Range(range) => {
                    range.start
                }
            };

            let new_time = time.round_subsecs(1).with_second(0).unwrap().with_minute(0).unwrap();
            match hour_events.get_mut(&new_time) {
                Some(v) => {
                    *v+=1;
                }
                None => {
                    hour_events.insert(new_time, 1);
                }
            }
        }

        let mut res: Vec<_> = hour_events.into_iter().map(|(time, amount)| Marker {time, amount}).collect();

        res.sort_by(|a, b| a.amount.cmp(&b.amount));
        res = res.into_iter().enumerate().filter(|(index, _elem)| index < &5).map(|(_index, elem)| elem).collect();

        Ok(res)
    }
    #[post("/markers", data="<request>")]
    pub async fn get_markers_request(request: Json<TimeRange>, config: &State<Config>, database: &State<Arc<Database>>, cookies: &CookieJar<'_>) -> status::Custom<Json<APIResult<Vec<Marker>>>> {
        let pwd = match cookies.get("pwd") {
            Some(v) => v,
            None => return status::Custom(Status::Unauthorized, Json(Err(APIError::AuthenticationError)))
        };
        if pwd.value() != config.password {
            status::Custom(Status::Unauthorized, Json(Err(APIError::AuthenticationError)))
        }
        else {
            status::Custom(Status::Ok, Json(get_markers(&request, database).await))
        }
    }
}

impl From<DatabaseError> for APIError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(format!("{}", value))
    }
}