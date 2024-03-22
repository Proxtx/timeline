use chrono::DateTime;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use std::fmt;

include!(concat!(env!("OUT_DIR"), "/plugins.rs"));

pub type APIResult<T: Serialize + DeserializeOwned> = Result<T, APIError>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum APIError {
    DatabaseError(String),
    AuthenticationError,
    #[cfg(feature = "client")]
    RequestError(String),
    SerdeJsonError(String),
    PluginError(String),
    Custom(String),
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
            #[cfg(feature = "client")]
            Self::RequestError(str) => {
                write!(f, "Request Error: {}", str)
            }
            Self::SerdeJsonError(txt) => {
                write!(f, "Error converting data to/from json: {}", txt)
            }
            Self::PluginError(txt) => {
                write!(
                    f,
                    "Error executing API Request. Encountered a plugin error: {}",
                    txt
                )
            }
            Self::Custom(txt) => {
                write!(f, "API Error: {}", txt)
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

#[derive(Serialize)]
#[cfg_attr(feature = "client", derive(Deserialize, Debug, Clone, PartialEq))]
pub struct CompressedEvent {
    #[serde(serialize_with = "serialize_data")]
    #[cfg(feature = "server")]
    pub data: Box<dyn erased_serde::Serialize + Sync + Send>,
    #[cfg(feature = "client")]
    pub data: String,
    pub time: crate::timing::Timing,
    pub title: String,
}

pub fn serialize_data<S>(
    data: &Box<dyn erased_serde::Serialize + Sync + Send>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&match serde_json::to_string(data) {
        Ok(v) => v,
        Err(e) => return Err(serde::ser::Error::custom(format!("{}", e))),
    })
}
