use chrono::DateTime;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

pub type APIResult<T: Serialize + DeserializeOwned> = Result<T, APIError>;

#[derive(Debug, Serialize, Deserialize)]
pub enum APIError {
    DatabaseError(String),
    AuthenticationError,
    #[cfg(feature = "reqwest")]
    RequestError(String),
    SerdeJsonError(String),
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => {
                write!(f, "Error executing API Request. Database Error: {}", e)
            }
            Self::AuthenticationError => {
                write!(
                    f,
                    "Error execution API Request: Authentication Error: Password is wrong"
                )
            }
            #[cfg(feature = "reqwest")]
            Self::RequestError(str) => {
                write!(f, "Request Error: {}", str)
            }
            Self::SerdeJsonError(txt) => {
                write!(f, "Error converting data to/from json: {}", txt)
            }
        }
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for APIError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::DatabaseError(format!("{}", value))
    }
}

#[cfg(feature = "reqwest")]
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
