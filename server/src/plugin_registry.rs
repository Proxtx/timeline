//! Talks to the configured plugin processes over HTTP.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use types::api::{APIError, APIResult, CompressedEvent};
use types::timing::TimeRange;

use crate::config::PluginEntry;

#[derive(Debug, Clone)]
pub struct PluginRegistry {
    plugins: Vec<PluginHandle>,
    by_name: HashMap<String, usize>,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct PluginHandle {
    pub name: String,
    pub base_url: url::Url,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteManifest {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub style: serde_json::Value,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub web_entry: Option<String>,
}

impl PluginRegistry {
    pub fn new(entries: &[PluginEntry]) -> Self {
        let plugins = entries
            .iter()
            .map(|e| PluginHandle {
                name: e.name.clone(),
                base_url: e.url.clone(),
                token: e.token.clone(),
            })
            .collect::<Vec<_>>();
        let by_name = plugins
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("build http client");
        PluginRegistry {
            plugins,
            by_name,
            client,
        }
    }

    pub fn all(&self) -> &[PluginHandle] {
        &self.plugins
    }

    pub fn get(&self, name: &str) -> Option<&PluginHandle> {
        self.by_name.get(name).and_then(|&i| self.plugins.get(i))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Fan-out `/events` to every plugin in parallel. Returns per-plugin
    /// event lists; plugins that fail are omitted and the error is logged.
    pub async fn fan_out_events(
        &self,
        range: &TimeRange,
    ) -> HashMap<String, Vec<CompressedEvent>> {
        use futures::stream::{FuturesUnordered, StreamExt};
        let me = Arc::new(self.clone());
        let mut futs = FuturesUnordered::new();
        for plugin in self.plugins.iter().cloned() {
            let me = me.clone();
            let range = range.clone();
            futs.push(async move {
                let name = plugin.name.clone();
                let result = me.events_for(&plugin, &range).await;
                (name, result)
            });
        }
        let mut out: HashMap<String, Vec<CompressedEvent>> = HashMap::new();
        while let Some((name, result)) = futs.next().await {
            match result {
                Ok(events) => {
                    out.insert(name, events);
                }
                Err(e) => {
                    tracing::warn!(plugin = %name, "events fetch failed: {}", e);
                }
            }
        }
        out
    }

    async fn events_for(
        &self,
        plugin: &PluginHandle,
        range: &TimeRange,
    ) -> APIResult<Vec<CompressedEvent>> {
        let url = plugin.base_url.join("events").map_err(|e| {
            APIError::Custom(format!("plugin {} bad url: {}", plugin.name, e))
        })?;
        let res = self
            .client
            .post(url)
            .bearer_auth(&plugin.token)
            .json(range)
            .send()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        if !status.is_success() {
            return Err(APIError::PluginError(format!(
                "{} returned {}: {}",
                plugin.name, status, text
            )));
        }
        serde_json::from_str::<APIResult<Vec<CompressedEvent>>>(&text)?
    }

    /// Collect the manifest shapes each plugin advertises.
    pub async fn fan_out_manifests(&self) -> Vec<RemoteManifest> {
        use futures::stream::{FuturesUnordered, StreamExt};
        let me = Arc::new(self.clone());
        let mut futs = FuturesUnordered::new();
        for plugin in self.plugins.iter().cloned() {
            let me = me.clone();
            futs.push(async move {
                let result = me.manifest_for(&plugin).await;
                (plugin.name.clone(), result)
            });
        }
        let mut out = Vec::new();
        while let Some((name, result)) = futs.next().await {
            match result {
                Ok(m) => out.push(m),
                Err(e) => tracing::warn!(plugin = %name, "manifest fetch failed: {}", e),
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    async fn manifest_for(&self, plugin: &PluginHandle) -> APIResult<RemoteManifest> {
        let url = plugin.base_url.join("manifest").map_err(|e| {
            APIError::Custom(format!("plugin {} bad url: {}", plugin.name, e))
        })?;
        let res = self
            .client
            .get(url)
            .bearer_auth(&plugin.token)
            .send()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| APIError::RequestError(e.to_string()))?;
        if !status.is_success() {
            return Err(APIError::PluginError(format!(
                "{} manifest returned {}",
                plugin.name, status
            )));
        }
        Ok(serde_json::from_str::<RemoteManifest>(&text)?)
    }
}
