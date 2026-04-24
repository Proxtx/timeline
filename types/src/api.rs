use serde::{Deserialize, Serialize};
use std::fmt;

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum APIError {
    DatabaseError(String),
    AuthenticationError,
    RequestError(String),
    SerdeJsonError(String),
    PluginError(String),
    Custom(String),
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::AuthenticationError => write!(f, "Authentication error"),
            Self::RequestError(e) => write!(f, "Request error: {}", e),
            Self::SerdeJsonError(e) => write!(f, "JSON error: {}", e),
            Self::PluginError(e) => write!(f, "Plugin error: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl From<serde_json::Error> for APIError {
    fn from(v: serde_json::Error) -> Self {
        Self::SerdeJsonError(v.to_string())
    }
}

/// Wire format for an event handed from plugin → main server → frontend.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompressedEvent {
    pub data: serde_json::Value,
    pub time: crate::timing::Timing,
    pub title: String,
}
