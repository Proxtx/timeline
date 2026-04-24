//! Blob storage for plugins.
//!
//! Plugins drop blobs under `<data_dir>/plugins/<name>/assets/<uuid>.<ext>`
//! and reference them from their event `data` by relative path. The SDK
//! mounts `GET /assets/<path>` so the main frontend can fetch them.

use std::path::{Path, PathBuf};

use tokio::fs;
use uuid::Uuid;

#[derive(Clone)]
pub struct AssetStore {
    root: PathBuf,
}

impl AssetStore {
    pub async fn open(root: impl Into<PathBuf>) -> Result<Self, AssetError> {
        let root = root.into();
        fs::create_dir_all(&root).await?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Write `bytes` as a fresh asset; return the relative path
    /// (e.g. `"e3f…a1.jpg"`) plugins should embed in their event data.
    pub async fn put(&self, bytes: &[u8], ext: &str) -> Result<String, AssetError> {
        let ext = ext.trim_start_matches('.');
        let name = if ext.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            format!("{}.{}", Uuid::new_v4(), ext)
        };
        let target = self.root.join(&name);
        fs::write(&target, bytes).await?;
        Ok(name)
    }

    /// Write `bytes` under a caller-chosen stable name (overwrites).
    /// Useful for migrations where the id is meaningful.
    pub async fn put_named(&self, name: &str, bytes: &[u8]) -> Result<String, AssetError> {
        guard_relative(name)?;
        let target = self.root.join(name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&target, bytes).await?;
        Ok(name.to_string())
    }

    pub async fn read(&self, rel: &str) -> Result<Vec<u8>, AssetError> {
        guard_relative(rel)?;
        Ok(fs::read(self.root.join(rel)).await?)
    }

    pub fn path_of(&self, rel: &str) -> Result<PathBuf, AssetError> {
        guard_relative(rel)?;
        Ok(self.root.join(rel))
    }

    pub async fn delete(&self, rel: &str) -> Result<(), AssetError> {
        guard_relative(rel)?;
        match fs::remove_file(self.root.join(rel)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

fn guard_relative(rel: &str) -> Result<(), AssetError> {
    if rel.is_empty() || rel.starts_with('/') || rel.contains("..") {
        return Err(AssetError::InvalidPath(rel.to_string()));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid asset path: {0}")]
    InvalidPath(String),
}
