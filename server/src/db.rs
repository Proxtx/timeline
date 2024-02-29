use {
    crate::AvailablePlugins,
    chrono::{DateTime, Utc},
    futures::StreamExt,
    mongodb::{
        bson::doc, error::Error as MongoDBError, Client, Collection, Database as MongoDatabase,
    },
    serde::{Deserialize, Serialize},
    std::fmt,
};

pub struct Database {
    database: MongoDatabase,
}

impl Database {
    pub async fn new(connection_string: &str, database: &str) -> DatabaseResult<Database> {
        let client = Client::with_uri_str(connection_string).await?;
        let database = client.database(database);

        Ok(Database { database })
    }

    pub async fn register_event<T>(&self, event: &Event<T>) -> DatabaseResult<()>
    where
        T: Serialize,
    {
        self.database
            .collection::<Event<T>>("events")
            .insert_one(event, None)
            .await?;
        Ok(())
    }

    pub fn get_events<T>(&self) -> Collection<Event<T>> {
        self.database.collection::<Event<T>>("events")
    }

    pub async fn event_count(&self) -> DatabaseResult<usize> {
        Ok(self
            .get_events::<mongodb::bson::Document>()
            .find(None, None)
            .await?
            .count()
            .await
            .to_le())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event<T> {
    timing: Timing,
    id: String,
    plugin: AvailablePlugins,
    event: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Timing {
    Range(DateTime<Utc>, DateTime<Utc>),
    Instant(DateTime<Utc>),
}

type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug)]
pub enum DatabaseError {
    SerializationError(mongodb::bson::ser::Error),
    MongoDBError(MongoDBError),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::SerializationError(e) => {
                write!(f, "Unable to serialize some data: {}", e)
            }
            DatabaseError::MongoDBError(e) => write!(f, "A Mongodb Database Error ocurred: {}", e),
        }
    }
}

impl From<MongoDBError> for DatabaseError {
    fn from(value: MongoDBError) -> Self {
        DatabaseError::MongoDBError(value)
    }
}

impl From<mongodb::bson::ser::Error> for DatabaseError {
    fn from(value: mongodb::bson::ser::Error) -> Self {
        DatabaseError::SerializationError(value)
    }
}
