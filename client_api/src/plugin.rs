pub trait Plugin {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_component(&self, data: PluginEventData) -> EventResult<Box<dyn FnOnce() -> View>>;

    fn get_style(&self) -> Style;

    fn get_icon(&self) -> IconLocation {
        IconLocation::Default
    }

    fn get_events_overview(&self, events: &Vec<CompressedEvent>) -> Option<View> {
        None
    }
}

pub enum IconLocation {
    Default,
    Custom(Url),
}

pub struct PluginEventData<'a> {
    data: &'a str,
}

impl<'a> PluginEventData<'a> {
    pub fn get_data<T>(&self) -> EventResult<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_str(self.data)?)
    }

    pub fn get_raw(&self) -> &str {
        self.data
    }
}

pub type EventResult<T> = Result<T, EventError>;

#[derive(Debug, Clone)]
pub enum EventError {
    FaultyInitData(String),
}

impl std::error::Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FaultyInitData(v) => {
                write!(
                    f,
                    "Unable to parse initial data to generate Component: {}",
                    v
                )
            }
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(value: serde_json::Error) -> Self {
        Self::FaultyInitData(format!("{}", value))
    }
}
