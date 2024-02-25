use serde::{de::DeserializeOwned, Serialize};

use crate::Plugin;
use std::fmt;
use std::fs::read_to_string;

pub struct Cache<CacheType>
where
    CacheType: Serialize + DeserializeOwned,
{
    cache: CacheType,
}

impl<CacheType> Cache<CacheType>
where
    CacheType: Serialize + DeserializeOwned,
{
    async fn load<PluginType>() -> CacheResult<Cache<CacheType>>
    where
        PluginType: Plugin,
    {
        let str = std::fs::read_to_string(format!("cache/{}", PluginType::get_type()))?;
        let t: CacheType = serde_json::from_str(&str)?;
        Ok(Cache { cache: t })
    }

    async fn update<PluginType>(&mut self, data: CacheType) -> CacheResult<()>
    where
        PluginType: Plugin,
    {
        let str = serde_json::to_string(&data)?;
        self.cache = data;
        std::fs::write(format!("cache/{}", PluginType::get_type()), str)?;
        Ok(())
    }
}

pub type CacheResult<T> = Result<T, CacheError>;

#[derive(Debug)]
pub enum CacheError {
    FileSystemError(std::io::Error),
    ParsingError(serde_json::Error),
}

impl std::error::Error for CacheError {}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::FileSystemError(e) => write!(f, "Unable to read/write cache: {}", e),
            CacheError::ParsingError(e) => write!(f, "Unable to parse cache: {}", e),
        }
    }
}

impl From<std::io::Error> for CacheError {
    fn from(value: std::io::Error) -> Self {
        CacheError::FileSystemError(value)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(value: serde_json::Error) -> Self {
        CacheError::ParsingError(value)
    }
}
