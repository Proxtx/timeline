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
        }
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for APIError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::DatabaseError(format!("{}", value))
    }
}
