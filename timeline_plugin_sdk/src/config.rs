//! Plugin configuration.
//!
//! A plugin's `config.toml` has a fixed `[plugin]` section consumed by the SDK
//! and an open-ended `[config]` section that the plugin's own `Plugin::new`
//! deserializes into whatever shape it wants.

use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct BaseConfig {
    pub plugin: PluginConfig,
    #[serde(default = "empty_value")]
    pub config: toml::Value,
}

fn empty_value() -> toml::Value {
    toml::Value::Table(toml::value::Table::new())
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    #[serde(default = "default_display_name")]
    pub display_name: Option<String>,
    pub port: u16,
    pub token: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub error_report_url: Option<Url>,
}

fn default_display_name() -> Option<String> {
    None
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

impl BaseConfig {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let raw = tokio::fs::read_to_string(path.as_ref()).await?;
        let cfg: BaseConfig = toml::from_str(&raw)?;
        Ok(cfg)
    }

    pub fn deserialize_plugin_config<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let cloned = self.config.clone();
        cloned.try_into().map_err(ConfigError::Toml)
    }
}

impl PluginConfig {
    pub fn plugin_root(&self) -> PathBuf {
        self.data_dir.join("plugins").join(&self.name)
    }

    pub fn db_path(&self) -> PathBuf {
        self.plugin_root().join("events.db")
    }

    pub fn assets_root(&self) -> PathBuf {
        self.plugin_root().join("assets")
    }

    pub fn cache_root(&self) -> PathBuf {
        self.plugin_root().join("cache")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("parsing config file: {0}")]
    Toml(#[from] toml::de::Error),
}
