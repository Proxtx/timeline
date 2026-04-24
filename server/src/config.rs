//! `config.toml` shape for the main timeline server.

use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub password: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub error_report_url: Option<Url>,
    #[serde(default)]
    pub plugin: Vec<PluginEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginEntry {
    /// Must match the plugin's own `plugin.name` — used in URLs
    /// (`/api/plugin/<name>/...`, `/plugin_web/<name>/...`) and as the key
    /// in the `/api/events` fan-out response.
    pub name: String,
    /// Base URL the main server contacts the plugin on
    /// (e.g. `http://127.0.0.1:9001`).
    pub url: Url,
    /// Shared bearer token sent to the plugin.
    pub token: String,
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

impl Config {
    pub async fn load(path: &str) -> Result<Self, ConfigError> {
        let raw = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&raw)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),
}
