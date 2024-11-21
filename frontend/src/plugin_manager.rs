use {
    crate::Plugins,
    leptos::View,
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt},
    types::api::{AvailablePlugins, CompressedEvent},
    url::Url,
};



impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Acc1 => {
                write!(f, "var(--accentColor1)")
            }
            Style::Acc2 => {
                write!(f, "var(--accentColor2)")
            }
            Style::Light => {
                write!(f, "var(--lighterColor)")
            }
            Style::Dark => {
                write!(f, "var(--darkColorLight)")
            }
            Style::Custom(dark_color, _, _) => write!(f, "{}", dark_color),
        }
    }
}

pub struct PluginData {}

#[derive(Clone)]
pub struct PluginManager {
    plugins: HashMap<AvailablePlugins, std::rc::Rc<Box<dyn Plugin>>>,
}

impl PluginManager {
    pub async fn new() -> Self {
        let plugins = Plugins::init(|_plugin| PluginData {}).await;

        PluginManager {
            plugins: plugins
                .plugins
                .into_iter()
                .map(|(k, v)| (k, std::rc::Rc::new(v)))
                .collect(),
        }
    }

    pub fn get_component(
        &self,
        plugin: &AvailablePlugins,
        data: &str,
    ) -> EventResult<impl FnOnce() -> View> {
        self.plugins
            .get(plugin)
            .unwrap()
            .get_component(PluginEventData { data })
    }

    pub fn get_style(&self, plugin: &AvailablePlugins) -> Style {
        self.plugins.get(plugin).unwrap().get_style()
    }

    pub fn get_icon(&self, plugin: &AvailablePlugins) -> IconLocation {
        self.plugins.get(plugin).unwrap().get_icon()
    }

    pub fn get_events_overview(&self, plugin: &AvailablePlugins, events: &Vec<CompressedEvent>) -> Option<View>
    {
        self.plugins.get(plugin).unwrap().get_events_overview(events)
    }
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

pub enum IconLocation {
    Default,
    Custom(Url),
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
