use {
    crate::AvailablePlugins,
    chrono::{DateTime, Utc},
    futures::StreamExt,
    mongodb::{
        bson::doc, error::Error as MongoDBError, results::InsertManyResult, Client, Collection,
        Database as MongoDatabase,
    },
    serde::{de::Visitor, Deserialize, Serialize},
    std::{
        borrow::BorrowMut,
        collections::HashMap,
        fmt::{self, format, Write},
        str::FromStr,
    },
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

    pub async fn register_single_event<T>(&self, event: &Event<T>) -> DatabaseResult<()>
    where
        T: Serialize,
    {
        self.database
            .collection::<Event<T>>("events")
            .insert_one(event, None)
            .await?;
        Ok(())
    }

    pub async fn register_events<T>(&self, events: &Vec<Event<T>>) -> DatabaseResult<()>
    where
        T: Serialize,
    {
        self.database
            .collection::<Event<T>>("events")
            .insert_many(events, None)
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Event<T> {
    pub timing: Timing,
    pub id: String,
    pub plugin: AvailablePlugins,
    pub event: T,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Timing {
    Range(DateTime<Utc>, DateTime<Utc>),
    Instant(DateTime<Utc>),
}

impl Serialize for Timing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let res = match self {
            Timing::Instant(t) => {
                vec![t.timestamp_millis()]
            }
            Timing::Range(start, end) => {
                vec![start.timestamp_millis(), end.timestamp_millis()]
            }
        };
        serializer.collect_seq(res)
    }
}

impl<'de> Deserialize<'de> for Timing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TimingVisitor)
    }
}

struct TimingVisitor;

impl<'de> Visitor<'de> for TimingVisitor {
    type Value = Timing;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list of either 1 or 2 values indicating a range or instant")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut seq_2: Vec<DateTime<Utc>> = vec![];
        while let Some(val) = seq.next_element()? {
            seq_2.push(match DateTime::from_timestamp_millis(val) {
                Some(v) => v,
                None => {
                    return Err(serde::de::Error::custom(
                        "Unable to parse milliseconds into DateTime: {}",
                    ))
                }
            });
        }

        match seq_2.len() {
            1 => Ok(Timing::Instant(seq_2[0])),
            2 => Ok(Timing::Range(seq_2[0], seq_2[1])),
            _ => Err(serde::de::Error::custom(
                "Unable to parse timing, since too many or too few values were provided",
            )),
        }
    }
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
