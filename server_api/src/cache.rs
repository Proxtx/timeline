use {
    crate::plugin::PluginTrait,
    serde::{de::DeserializeOwned, Serialize},
    std::fmt,
    tokio::fs::read_to_string,
    types::external::serde_json,
};

#[allow(unused)]
pub struct Cache<CacheType>
where
    CacheType: Serialize + DeserializeOwned + Default,
{
    cache: CacheType,
}

#[allow(unused)]
impl<CacheType> Cache<CacheType>
where
    CacheType: Serialize + DeserializeOwned + Default,
{
    pub async fn load<'a, PluginType>() -> CacheResult<Cache<CacheType>>
    where
        PluginType: PluginTrait,
    {
        match read_to_string(format!("cache/{}", PluginType::get_type())).await {
            Ok(str) => {
                let t: CacheType = serde_json::from_str(&str)?;
                Ok(Cache { cache: t })
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(Cache {
                    cache: CacheType::default(),
                }),
                _ => Err(CacheError::FileSystemError(e)),
            },
        }
    }

    pub fn get(&self) -> &CacheType {
        &self.cache
    }

    pub fn modify<PluginType>(&mut self, updater: impl FnOnce(&mut CacheType)) -> CacheResult<()>
    where
        PluginType: PluginTrait,
    {
        updater(&mut self.cache);
        self.save::<PluginType>()
    }

    pub fn update<PluginType>(&mut self, data: CacheType) -> CacheResult<()>
    where
        PluginType: PluginTrait,
    {
        self.cache = data;
        self.save::<PluginType>()?;
        Ok(())
    }

    pub fn save<PluginType>(&self) -> CacheResult<()>
    where
        PluginType: PluginTrait,
    {
        let str = serde_json::to_string(&self.cache)?;

        tokio::spawn(async move {
            if let Err(e) = std::fs::write(format!("cache/{}", PluginType::get_type()), str) {
                eprintln!("Unable to write cache file: {}", e)
            }
        });
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
