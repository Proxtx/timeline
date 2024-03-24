use {
    crate::{event_manager::EventResult, Plugins},
    leptos::View,
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt},
    types::api::AvailablePlugins,
    url::Url,
};
pub trait Plugin {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_component(&self, data: PluginEventData) -> EventResult<Box<dyn FnOnce() -> View>>;

    fn get_style(&self) -> Style;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Acc1,
    Acc2,
}

impl Style {
    pub fn light(&self) -> &'static str {
        match self {
            Style::Acc1 => "var(--accentColor1Light)",
            Style::Acc2 => "var(--accentColor2Light)",
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Acc1 => {
                write!(f, "var(--accentColor1)")
            }
            Style::Acc2 => {
                write!(f, "var(--accentColor2)")
            }
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

    pub fn get_icon(&self) -> IconLocation {
        IconLocation::Default
    }
}

pub enum IconLocation {
    Default,
    Custom(Url),
}
