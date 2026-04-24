//! Timeline plugin SDK.
//!
//! A timeline plugin is a standalone Rocket binary that the main timeline
//! server talks to over HTTP. Every plugin author implements the [`Plugin`]
//! trait and calls [`launch`] from `main`; the SDK takes care of config
//! loading, SQLite-backed event storage, asset storage, bearer-token auth,
//! and the standard HTTP contract (`/events`, `/manifest`, `/assets/<path>`,
//! `/health`).

pub mod assets;
pub mod auth;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod launch;
pub mod manifest;
pub mod plugin;
pub mod routes;

pub use assets::AssetStore;
pub use cache::Cache;
pub use config::{BaseConfig, PluginConfig};
pub use db::{Db, StoredEvent};
pub use error::ErrorReporter;
pub use launch::launch;
pub use manifest::{Manifest, Style};
pub use plugin::{Context, Plugin};

pub use types::api::{APIError, APIResult, CompressedEvent};
pub use types::timing::{TimeRange, Timing};

pub use rocket;
pub use serde_json;
pub use tokio;
