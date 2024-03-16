use crate::db::{Database, DatabaseError, Event};
use chrono::{DateTime, SubsecRound, Timelike, Utc};
use futures::StreamExt;
use mongodb::{bson::{doc, Document}, options::FindOptions};
use serde::Deserialize;
use std::{collections::HashMap, fmt};
use std::sync::Arc;
use types::{Marker, TimeRange, Timing};

pub async fn get_markers(range: &TimeRange, database: Arc<Database>) -> APIResult<Vec<Marker>> {
    #[derive(Deserialize)]
    struct OnlyTimingEvent {
        timing: Timing
    }
    
    let mut events = database
        .find_events_with_custom_query::<OnlyTimingEvent>(Database::generate_range_filter(range), FindOptions::builder().sort(doc! {
            "timing.0": 1
        }).build()).await?;
    
    let mut hour_events: HashMap<DateTime<Utc>, u32> = HashMap::new();

    while events.advance().await? {
        let next_event = events.deserialize_current()?;
        let time = match next_event.timing {
            Timing::Instant(t) => {
                t
            }
            Timing::Range(range) => {
                range.start
            }
        };

        let new_time = time.round_subsecs(1).with_second(0).unwrap().with_minute(0).unwrap();
        match hour_events.get_mut(&new_time) {
            Some(v) => {
                *v+=1;
            }
            None => {
                hour_events.insert(new_time, 1);
            }
        }
    }

    let mut res: Vec<_> = hour_events.into_iter().map(|(time, amount)| Marker {time, amount}).collect();

    res.sort_by(|a, b| a.amount.cmp(&b.amount));
    res = res.into_iter().enumerate().filter(|(index, _elem)| index < &5).map(|(_index, elem)| elem).collect();

    Ok(res)
}

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug)]
pub enum APIError {
    DatabaseError(DatabaseError),
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => {
                write!(f, "Error executing API Request. Database Error: {}", e)
            }
        }
    }
}

impl From<DatabaseError> for APIError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<mongodb::error::Error> for APIError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::DatabaseError(DatabaseError::MongoDBError(value))
    }
}
