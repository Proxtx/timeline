use crate::db::{Database, DatabaseError};
use mongodb::bson::Document;
use std::fmt;
use std::sync::Arc;
use types::{Marker, TimeRange};

/*pub fn get_markers(range: TimeRange, database: Arc<Database>) -> Vec<Marker> {
    database.get_events::<Document>().find(, options)
}*/

#[derive(Debug)]
enum APIError {
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
