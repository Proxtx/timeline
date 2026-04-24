//! Forward `/api/plugin/<name>/<path..>` requests to the plugin process,
//! adding the bearer token the plugin expects.

use std::path::PathBuf;

use reqwest::Method;
use rocket::data::{ByteUnit, Data};
use rocket::http::{ContentType, Status};
use rocket::{delete, get, post, put, response, State};

use crate::plugin_registry::PluginRegistry;

/// 8 MiB should comfortably cover plugin bodies (event registration, etc).
const MAX_BODY_BYTES: ByteUnit = ByteUnit::Mebibyte(8);

pub struct ProxiedResponse {
    status: Status,
    content_type: Option<ContentType>,
    body: Vec<u8>,
}

impl<'r> response::Responder<'r, 'static> for ProxiedResponse {
    fn respond_to(self, _req: &'r rocket::Request<'_>) -> response::Result<'static> {
        let mut builder = response::Response::build();
        builder.status(self.status);
        if let Some(ct) = self.content_type {
            builder.header(ct);
        }
        builder.sized_body(self.body.len(), std::io::Cursor::new(self.body));
        builder.ok()
    }
}

#[get("/plugin/<name>/<path..>")]
pub async fn proxy_get(
    name: &str,
    path: PathBuf,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    proxy(registry, name, path, Method::GET, None).await
}

#[post("/plugin/<name>/<path..>", data = "<data>")]
pub async fn proxy_post(
    name: &str,
    path: PathBuf,
    data: Data<'_>,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let body = data
        .open(MAX_BODY_BYTES)
        .into_bytes()
        .await
        .map_err(|_| Status::PayloadTooLarge)?
        .value;
    proxy(registry, name, path, Method::POST, Some(body)).await
}

#[put("/plugin/<name>/<path..>", data = "<data>")]
pub async fn proxy_put(
    name: &str,
    path: PathBuf,
    data: Data<'_>,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let body = data
        .open(MAX_BODY_BYTES)
        .into_bytes()
        .await
        .map_err(|_| Status::PayloadTooLarge)?
        .value;
    proxy(registry, name, path, Method::PUT, Some(body)).await
}

#[delete("/plugin/<name>/<path..>")]
pub async fn proxy_delete(
    name: &str,
    path: PathBuf,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    proxy(registry, name, path, Method::DELETE, None).await
}

async fn proxy(
    registry: &PluginRegistry,
    name: &str,
    path: PathBuf,
    method: Method,
    body: Option<Vec<u8>>,
) -> Result<ProxiedResponse, Status> {
    let plugin = registry.get(name).ok_or(Status::NotFound)?;
    let tail = path.to_string_lossy();
    let upstream = plugin
        .base_url
        .join(&tail)
        .map_err(|_| Status::BadRequest)?;

    let mut req = registry
        .client()
        .request(method, upstream)
        .bearer_auth(&plugin.token);
    if let Some(body) = body {
        req = req.body(body);
    }
    let res = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(plugin = %name, "proxy failed: {}", e);
            return Err(Status::BadGateway);
        }
    };
    let status = Status::from_code(res.status().as_u16()).unwrap_or(Status::InternalServerError);
    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| ContentType::parse_flexible(s));
    let body = res
        .bytes()
        .await
        .map_err(|_| Status::BadGateway)?
        .to_vec();

    Ok(ProxiedResponse {
        status,
        content_type,
        body,
    })
}
