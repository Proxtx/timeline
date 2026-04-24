//! HTTP client for the main timeline server's `/api/*` endpoints.
//! Replaces `client_api::api`.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use types::api::{APIError, APIResult};

#[derive(Clone, Debug)]
pub struct TimelineHostname(pub String);

/// POST JSON `request` to `/api{endpoint}` on the timeline server and
/// deserialize the response as `APIResult<T>`. Mirrors the old
/// `client_api::api::api_request`.
pub async fn api_request<T, V>(endpoint: &str, request: &V) -> APIResult<T>
where
    T: DeserializeOwned,
    V: Serialize,
{
    let url = relative_url(&format!("/api{}", endpoint));
    let body = serde_json::to_string(request).map_err(|e| APIError::SerdeJsonError(e.to_string()))?;

    let res = Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| APIError::RequestError(e.to_string()))?
        .send()
        .await
        .map_err(|e| APIError::RequestError(e.to_string()))?;

    let text = res
        .text()
        .await
        .map_err(|e| APIError::RequestError(e.to_string()))?;

    serde_json::from_str::<APIResult<T>>(&text)
        .map_err(|e| APIError::SerdeJsonError(e.to_string()))?
}

pub fn relative_url(path: &str) -> String {
    match use_context::<TimelineHostname>() {
        Some(h) => {
            let base = h.0.trim_end_matches('/');
            if path.starts_with('/') {
                format!("{}{}", base, path)
            } else {
                format!("{}/{}", base, path)
            }
        }
        None => path.to_string(),
    }
}
