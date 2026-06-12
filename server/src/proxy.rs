//! Forward `/api/plugin/<name>/<path..>` requests to the plugin process,
//! adding the bearer token the plugin expects.

use reqwest::Method;
use rocket::data::{ByteUnit, Data};
use rocket::http::uri::fmt::Path as UriPath;
use rocket::http::uri::Segments;
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::{delete, get, post, put, response, Request, State};

use crate::plugin_registry::PluginRegistry;

/// 8 MiB should comfortably cover plugin bodies (event registration, etc).
const MAX_BODY_BYTES: ByteUnit = ByteUnit::Mebibyte(8);

/// The raw, still-percent-encoded path + query of the incoming request.
///
/// We can't reconstruct the upstream URL from the `<path..>` `PathBuf`: signed
/// file URLs (documents / media_scan) encode an absolute file path as a single
/// segment containing `%2F`, and the trailing segment carries a base64
/// signature with `%2F`, `%2B`, `%3D`. Rocket's `PathBuf` segment guard rejects
/// decoded `/`, so those requests never match and fall through to the SPA 404
/// catcher. Even if they matched, decoding would corrupt the bytes the plugin
/// signed over. So we forward the raw tail verbatim.
struct RawTail {
    path: String,
    query: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawTail {
    type Error = std::convert::Infallible;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(RawTail {
            path: req.uri().path().as_str().to_string(),
            query: req.uri().query().map(|q| q.as_str().to_string()),
        })
    }
}

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

// `_path` only exists to bind the route's `<path..>`; the actual tail comes
// from `RawTail` (the `Segments` guard, unlike `PathBuf`, doesn't reject `%2F`).
#[get("/plugin/<name>/<path..>")]
pub async fn proxy_get(
    name: &str,
    path: Segments<'_, UriPath>,
    raw: RawTail,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let _ = path;
    proxy(registry, name, raw, Method::GET, None).await
}

#[post("/plugin/<name>/<path..>", data = "<data>")]
pub async fn proxy_post(
    name: &str,
    path: Segments<'_, UriPath>,
    raw: RawTail,
    data: Data<'_>,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let _ = path;
    let body = data
        .open(MAX_BODY_BYTES)
        .into_bytes()
        .await
        .map_err(|_| Status::PayloadTooLarge)?
        .value;
    proxy(registry, name, raw, Method::POST, Some(body)).await
}

#[put("/plugin/<name>/<path..>", data = "<data>")]
pub async fn proxy_put(
    name: &str,
    path: Segments<'_, UriPath>,
    raw: RawTail,
    data: Data<'_>,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let _ = path;
    let body = data
        .open(MAX_BODY_BYTES)
        .into_bytes()
        .await
        .map_err(|_| Status::PayloadTooLarge)?
        .value;
    proxy(registry, name, raw, Method::PUT, Some(body)).await
}

#[delete("/plugin/<name>/<path..>")]
pub async fn proxy_delete(
    name: &str,
    path: Segments<'_, UriPath>,
    raw: RawTail,
    registry: &State<PluginRegistry>,
) -> Result<ProxiedResponse, Status> {
    let _ = path;
    proxy(registry, name, raw, Method::DELETE, None).await
}

async fn proxy(
    registry: &PluginRegistry,
    name: &str,
    raw: RawTail,
    method: Method,
    body: Option<Vec<u8>>,
) -> Result<ProxiedResponse, Status> {
    let plugin = registry.get(name).ok_or(Status::NotFound)?;
    // Strip the `/api/plugin/<name>/` prefix, keeping the raw encoding of the
    // tail. `Url::join`/`Url::parse` preserve existing `%XX` sequences in the
    // path, so the plugin receives the exact signed bytes back.
    let prefix = format!("/api/plugin/{}/", name);
    let tail = raw.path.strip_prefix(&prefix).unwrap_or("");
    let mut upstream = plugin
        .base_url
        .join(tail)
        .map_err(|_| Status::BadRequest)?;
    upstream.set_query(raw.query.as_deref());

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
