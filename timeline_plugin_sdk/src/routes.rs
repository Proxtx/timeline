//! Standard HTTP endpoints every plugin exposes.
//!
//! `/events`, `/manifest`, `/health`, and `/assets/<path..>` are identical
//! across every plugin. Plugin-specific routes come from [`Plugin::routes`]
//! and are mounted alongside.

use std::path::PathBuf;

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};

use crate::auth::AuthedClient;
use crate::launch::{PluginHandle, PluginState};
use types::api::{APIError, APIResult, CompressedEvent};
use types::timing::TimeRange;

#[post("/events", data = "<range>")]
pub async fn events(
    _auth: AuthedClient,
    range: Json<TimeRange>,
    handle: &State<PluginHandle>,
) -> Json<APIResult<Vec<CompressedEvent>>> {
    Json(handle.events(range.into_inner()).await)
}

#[get("/manifest")]
pub async fn manifest(
    _auth: AuthedClient,
    handle: &State<PluginHandle>,
) -> Json<crate::manifest::Manifest> {
    Json(handle.manifest())
}

#[get("/health")]
pub async fn health() -> &'static str {
    "ok"
}

#[get("/assets/<path..>")]
pub async fn assets(
    _auth: AuthedClient,
    path: PathBuf,
    state: &State<PluginState>,
) -> Result<NamedFile, Status> {
    let rel = path.to_string_lossy();
    match state.assets.path_of(&rel) {
        Ok(full) => NamedFile::open(full).await.map_err(|_| Status::NotFound),
        Err(_) => Err(Status::BadRequest),
    }
}

/// Convenience: turn an APIResult into a 500-or-200 for plugin authors who
/// want the SDK's standard error shape in custom routes.
pub fn wrap_api<T: serde::Serialize>(v: APIResult<T>) -> Result<Json<T>, (Status, String)> {
    match v {
        Ok(t) => Ok(Json(t)),
        Err(APIError::AuthenticationError) => Err((Status::Unauthorized, "auth".into())),
        Err(e) => Err((Status::InternalServerError, e.to_string())),
    }
}
