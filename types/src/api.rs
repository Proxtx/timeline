#[cfg(feature = "client")]
use crate::available_plugins::AvailablePlugins;
use {
    serde::{Deserialize, Serialize},
    std::fmt,
};

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum APIError {
    DatabaseError(String),
    AuthenticationError,
    #[cfg(feature = "client")]
    RequestError(String),
    SerdeJsonError(String),
    PluginError(String),
    Custom(String),
    #[cfg(feature = "experiences")]
    ExperienceError(String),
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => {
                write!(f, "Error executing API Request: Database Error: {}", e)
            }
            Self::AuthenticationError => {
                write!(
                    f,
                    "Error executing API Request: Authentication Error: Password is wrong"
                )
            }
            #[cfg(feature = "client")]
            Self::RequestError(str) => {
                write!(
                    f,
                    "Error executing API Request: HTTP-Request Error: {}",
                    str
                )
            }
            Self::SerdeJsonError(txt) => {
                write!(
                    f,
                    "Error executing API Request: Error converting data to/from json: {}",
                    txt
                )
            }
            Self::PluginError(txt) => {
                write!(
                    f,
                    "Error executing API Request: Encountered a plugin error: {}",
                    txt
                )
            }
            Self::Custom(txt) => {
                write!(f, "API Error: {}", txt)
            }
            #[cfg(feature = "experiences")]
            Self::ExperienceError(txt) => {
                write!(
                    f,
                    "Error executing API Request: Encountered an experience error: {}",
                    txt
                )
            }
        }
    }
}

#[cfg(feature = "server")]
impl From<mongodb::error::Error> for APIError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::DatabaseError(format!("{}", value))
    }
}

#[cfg(feature = "client")]
impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(format!("{}", value))
    }
}

impl From<serde_json::Error> for APIError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJsonError(format!("{}", value))
    }
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "client", derive(Deserialize, Clone, PartialEq))]
pub struct CompressedEvent {
    pub data: serde_json::Value,
    pub time: crate::timing::Timing,
    pub title: String,
}

#[cfg(feature = "client")]
pub trait EventWrapper
where
    Self: Clone + PartialEq + 'static,
{
    fn get_compressed_event(&self) -> CompressedEvent;
}

#[cfg(feature = "client")]
impl EventWrapper for CompressedEvent {
    fn get_compressed_event(&self) -> CompressedEvent {
        self.clone()
    }
}

#[cfg(feature = "client")]
impl EventWrapper for (AvailablePlugins, CompressedEvent) {
    fn get_compressed_event(&self) -> CompressedEvent {
        self.1.clone()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimelineHostname(pub String);
