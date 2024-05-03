use {
    serde::Deserialize,
    std::{collections::HashMap, fmt},
    tokio::{fs::File, io::AsyncReadExt},
    url::Url,
};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub password: String,
    pub port: u16,
    pub db_connection_string: String,
    pub database: String,
    pub plugin_config: HashMap<crate::AvailablePlugins, toml::Value>,
    pub error_report_url: Option<Url>,
}

impl Config {
    pub async fn load() -> ConfigResult<Config> {
        let mut config = String::new();
        File::open("config.toml")
            .await?
            .read_to_string(&mut config)
            .await?;
        Ok(toml::from_str::<Config>(&config)?)
    }
}

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    FileSystemError(std::io::Error),
    ParserError(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParserError(e) => {
                write!(f, "Error parsing config file: {}", e)
            }
            ConfigError::FileSystemError(e) => {
                write!(f, "Error reading config file: {}", e)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::FileSystemError(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::ParserError(value)
    }
}
