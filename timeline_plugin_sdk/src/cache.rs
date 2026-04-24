//! Simple JSON-on-disk cache. Same semantics as the old `server_api::Cache`
//! but re-rooted at `<data_dir>/plugins/<name>/cache/<key>.json` and without
//! the plugin-type generic parameter.

use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Cache {
    root: PathBuf,
    lock: std::sync::Arc<Mutex<()>>,
}

impl Cache {
    pub async fn open(root: impl Into<PathBuf>) -> Result<Self, CacheError> {
        let root = root.into();
        fs::create_dir_all(&root).await?;
        Ok(Self {
            root,
            lock: std::sync::Arc::new(Mutex::new(())),
        })
    }

    fn path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{}.json", key))
    }

    /// Load a cached value; returns `T::default()` when the file is missing.
    pub async fn load<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T, CacheError> {
        let _guard = self.lock.lock().await;
        match fs::read_to_string(self.path(key)).await {
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn save<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        let _guard = self.lock.lock().await;
        let data = serde_json::to_string(value)?;
        fs::write(self.path(key), data).await?;
        Ok(())
    }

    pub async fn modify<T, F>(&self, key: &str, f: F) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned + Default + Clone,
        F: FnOnce(&mut T),
    {
        let mut value = self.load::<T>(key).await?;
        f(&mut value);
        self.save(key, &value).await?;
        Ok(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
