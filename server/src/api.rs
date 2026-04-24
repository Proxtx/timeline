//! Core `/api/*` routes on the main server (non-proxied).

use std::collections::HashMap;

use chrono::{DateTime, SubsecRound, Timelike, Utc};
use rocket::http::{CookieJar, Status};
use rocket::post;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use types::api::{APIError, APIResult, CompressedEvent};
use types::timing::{Marker, TimeRange, Timing};

use crate::config::Config;
use crate::plugin_registry::{PluginRegistry, RemoteManifest};

// ---------- auth ----------

fn auth(cookies: &CookieJar<'_>, config: &Config) -> APIResult<()> {
    match cookies.get("pwd") {
        Some(c) if c.value() == config.password => Ok(()),
        _ => Err(APIError::AuthenticationError),
    }
}

#[post("/auth")]
pub fn auth_request(
    cookies: &CookieJar<'_>,
    config: &State<Config>,
) -> status::Custom<Json<APIResult<()>>> {
    status::Custom(Status::Ok, Json(auth(cookies, config)))
}

// ---------- events (fan-out) ----------

#[post("/events", data = "<range>")]
pub async fn events(
    range: Json<TimeRange>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
    registry: &State<PluginRegistry>,
) -> status::Custom<Json<APIResult<HashMap<String, Vec<CompressedEvent>>>>> {
    if let Err(e) = auth(cookies, config) {
        return status::Custom(Status::Unauthorized, Json(Err(e)));
    }
    let events = registry.fan_out_events(&range).await;
    status::Custom(Status::Ok, Json(Ok(events)))
}

// ---------- markers (derived from fan-out) ----------

#[post("/markers", data = "<range>")]
pub async fn markers(
    range: Json<TimeRange>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
    registry: &State<PluginRegistry>,
) -> status::Custom<Json<APIResult<Vec<Marker>>>> {
    if let Err(e) = auth(cookies, config) {
        return status::Custom(Status::Unauthorized, Json(Err(e)));
    }
    let events = registry.fan_out_events(&range).await;
    let markers = derive_markers(events);
    status::Custom(Status::Ok, Json(Ok(markers)))
}

fn derive_markers(all: HashMap<String, Vec<CompressedEvent>>) -> Vec<Marker> {
    let mut buckets: HashMap<DateTime<Utc>, u32> = HashMap::new();
    for (_plugin, events) in all {
        for e in events {
            let t = match e.time {
                Timing::Instant(t) => t,
                Timing::Range(r) => r.start,
            };
            let Some(hourly) = t
                .round_subsecs(0)
                .with_second(0)
                .and_then(|x| x.with_minute(0))
            else {
                continue;
            };
            *buckets.entry(hourly).or_insert(0) += 1;
        }
    }
    let mut out: Vec<_> = buckets
        .into_iter()
        .map(|(time, amount)| Marker { time, amount })
        .collect();
    out.sort_by(|a, b| b.amount.cmp(&a.amount));
    out
}

// ---------- manifest aggregation ----------

#[post("/plugins")]
pub async fn plugins(
    cookies: &CookieJar<'_>,
    config: &State<Config>,
    registry: &State<PluginRegistry>,
) -> status::Custom<Json<APIResult<Vec<RemoteManifest>>>> {
    if let Err(e) = auth(cookies, config) {
        return status::Custom(Status::Unauthorized, Json(Err(e)));
    }
    let manifests = registry.fan_out_manifests().await;
    status::Custom(Status::Ok, Json(Ok(manifests)))
}
