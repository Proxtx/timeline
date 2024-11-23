use std::fmt;
use types::external::serde_json;

pub type EventResult<T> = Result<T, EventError>;

#[derive(Debug, Clone)]
pub enum EventError {
    FaultyInitData(String),
}

impl std::error::Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FaultyInitData(v) => {
                write!(
                    f,
                    "Unable to parse initial data to generate Component: {}",
                    v
                )
            }
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(value: serde_json::Error) -> Self {
        Self::FaultyInitData(format!("{}", value))
    }
}
