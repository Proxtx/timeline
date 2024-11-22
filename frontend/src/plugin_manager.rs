use {
    crate::Plugins,
    leptos::View,
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt},
    types::api::{AvailablePlugins, CompressedEvent},
    url::Url,
};

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