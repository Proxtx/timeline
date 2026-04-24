//! Tiny HTTP client for plugins to talk to their own backend via the main
//! server's proxy (`/api/plugin/<name>/...`). The main frontend fills in
//! [`PluginContext::api_base`] so plugins don't need to know hostnames.

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};

use types::api::{APIError, APIResult};

#[derive(Debug, Clone)]
pub struct ApiClient {
    base: String,
}

impl ApiClient {
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            base: api_base.into(),
        }
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base.trim_end_matches('/'), path)
        } else {
            format!("{}/{}", self.base.trim_end_matches('/'), path)
        }
    }

    pub async fn post<Req: Serialize, Res: DeserializeOwned>(
        &self,
        path: &str,
        body: &Req,
    ) -> APIResult<Res> {
        let res = Request::post(&self.url(path))
            .credentials(web_sys::RequestCredentials::Include)
            .json(body)
            .map_err(|e| APIError::RequestError(e.to_string()))?
            .send()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        if !res.ok() {
            if res.status() == 401 {
                return Err(APIError::AuthenticationError);
            }
            return Err(APIError::RequestError(format!(
                "HTTP {}: {}",
                res.status(),
                res.status_text()
            )));
        }
        res.json::<Res>()
            .await
            .map_err(|e| APIError::SerdeJsonError(e.to_string()))
    }

    pub async fn get<Res: DeserializeOwned>(&self, path: &str) -> APIResult<Res> {
        let res = Request::get(&self.url(path))
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        if !res.ok() {
            if res.status() == 401 {
                return Err(APIError::AuthenticationError);
            }
            return Err(APIError::RequestError(format!(
                "HTTP {}: {}",
                res.status(),
                res.status_text()
            )));
        }
        res.json::<Res>()
            .await
            .map_err(|e| APIError::SerdeJsonError(e.to_string()))
    }

    /// Construct a fetch URL inside the plugin's proxy namespace. Useful
    /// for image `src` attributes etc.
    pub fn asset_url(&self, relative: &str) -> String {
        self.url(relative)
    }
}
