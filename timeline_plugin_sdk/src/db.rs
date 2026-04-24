//! SQLite-backed event store.
//!
//! One database per plugin. Events are indexed by `(start_ts, end_ts)`. The
//! payload is JSON-encoded so plugins can keep storing arbitrary
//! `serde_json::Value` shapes exactly like they did with MongoDB.

use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use types::api::{APIError, APIResult, CompressedEvent};
use types::timing::{TimeRange, Timing};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

/// Internal row shape. Plugins never see this — they hand us a
/// [`CompressedEvent`] + dedup id and we wrap it.
#[derive(Debug, Clone)]
pub struct StoredEvent<T = serde_json::Value> {
    pub id: String,
    pub title: String,
    pub time: Timing,
    pub data: T,
}

impl Db {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let opts = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (\n\
               id       TEXT PRIMARY KEY,\n\
               start_ts INTEGER NOT NULL,\n\
               end_ts   INTEGER NOT NULL,\n\
               title    TEXT NOT NULL,\n\
               data     TEXT NOT NULL\n\
             ) STRICT",
        )
        .execute(&pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS events_time_idx ON events(start_ts, end_ts)")
            .execute(&pool)
            .await?;

        Ok(Db { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Insert or replace one event. Plugin-supplied `id` is the dedup key.
    pub async fn upsert<T: Serialize>(&self, event: &StoredEvent<T>) -> Result<(), DbError> {
        let (start_ts, end_ts) = timing_bounds(&event.time);
        let data = serde_json::to_string(&event.data)?;
        sqlx::query(
            "INSERT INTO events (id, start_ts, end_ts, title, data) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               start_ts = excluded.start_ts, \
               end_ts   = excluded.end_ts, \
               title    = excluded.title, \
               data     = excluded.data",
        )
        .bind(&event.id)
        .bind(start_ts)
        .bind(end_ts)
        .bind(&event.title)
        .bind(&data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_many<T: Serialize>(
        &self,
        events: &[StoredEvent<T>],
    ) -> Result<(), DbError> {
        if events.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for event in events {
            let (start_ts, end_ts) = timing_bounds(&event.time);
            let data = serde_json::to_string(&event.data)?;
            sqlx::query(
                "INSERT INTO events (id, start_ts, end_ts, title, data) VALUES (?, ?, ?, ?, ?) \
                 ON CONFLICT(id) DO UPDATE SET \
                   start_ts = excluded.start_ts, \
                   end_ts   = excluded.end_ts, \
                   title    = excluded.title, \
                   data     = excluded.data",
            )
            .bind(&event.id)
            .bind(start_ts)
            .bind(end_ts)
            .bind(&event.title)
            .bind(&data)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM events WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<i64, DbError> {
        let row = sqlx::query("SELECT COUNT(*) as c FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<i64, _>("c")?)
    }

    /// Return all events whose timing overlaps `range`. Ordered by start time.
    pub async fn query_range(&self, range: &TimeRange) -> Result<Vec<CompressedEvent>, DbError> {
        let start = range.start.timestamp_millis();
        let end = range.end.timestamp_millis();

        // overlap condition: start_ts < range.end AND end_ts >= range.start
        let rows = sqlx::query(
            "SELECT id, start_ts, end_ts, title, data \
             FROM events \
             WHERE start_ts < ? AND end_ts >= ? \
             ORDER BY start_ts ASC",
        )
        .bind(end)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let start_ts: i64 = row.try_get("start_ts")?;
            let end_ts: i64 = row.try_get("end_ts")?;
            let title: String = row.try_get("title")?;
            let data: String = row.try_get("data")?;
            let data: serde_json::Value = serde_json::from_str(&data)?;
            let time = bounds_to_timing(start_ts, end_ts);
            out.push(CompressedEvent { data, time, title });
        }
        Ok(out)
    }

    /// Typed variant for plugins that want to read their own payloads back.
    pub async fn query_range_typed<T: DeserializeOwned>(
        &self,
        range: &TimeRange,
    ) -> Result<Vec<StoredEvent<T>>, DbError> {
        let start = range.start.timestamp_millis();
        let end = range.end.timestamp_millis();

        let rows = sqlx::query(
            "SELECT id, start_ts, end_ts, title, data \
             FROM events \
             WHERE start_ts < ? AND end_ts >= ? \
             ORDER BY start_ts ASC",
        )
        .bind(end)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.try_get("id")?;
            let start_ts: i64 = row.try_get("start_ts")?;
            let end_ts: i64 = row.try_get("end_ts")?;
            let title: String = row.try_get("title")?;
            let data: String = row.try_get("data")?;
            let data: T = serde_json::from_str(&data)?;
            out.push(StoredEvent {
                id,
                title,
                time: bounds_to_timing(start_ts, end_ts),
                data,
            });
        }
        Ok(out)
    }
}

fn timing_bounds(t: &Timing) -> (i64, i64) {
    match t {
        Timing::Instant(dt) => {
            let ms = dt.timestamp_millis();
            (ms, ms)
        }
        Timing::Range(r) => (r.start.timestamp_millis(), r.end.timestamp_millis()),
    }
}

fn bounds_to_timing(start_ts: i64, end_ts: i64) -> Timing {
    use chrono::{DateTime, Utc};
    let start = DateTime::<Utc>::from_timestamp_millis(start_ts).unwrap_or_default();
    if start_ts == end_ts {
        Timing::Instant(start)
    } else {
        let end = DateTime::<Utc>::from_timestamp_millis(end_ts).unwrap_or_default();
        Timing::Range(TimeRange { start, end })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<DbError> for APIError {
    fn from(value: DbError) -> Self {
        APIError::DatabaseError(value.to_string())
    }
}

/// Convenience for plugins implementing `Plugin::events`.
pub fn to_api_result<T>(v: Result<T, DbError>) -> APIResult<T> {
    v.map_err(|e| e.into())
}
