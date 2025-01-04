use {
    client_api::{
        external::{
            types::{api::CompressedEvent, available_plugins::AvailablePlugins},
        },
        plugin::{IconLocation, PluginData, PluginEventData, PluginTrait},
        result::EventResult,
        style::Style,
        types::external::serde_json,
    },
    dyn_link::client_plugins::Plugins,
    leptos::View,
    std::{collections::HashMap},
};

#[derive(Clone)]
pub struct PluginManager {
    plugins: HashMap<AvailablePlugins, std::rc::Rc<Box<dyn PluginTrait>>>,
}

impl PluginManager {
    pub async fn new() -> Self {
        let mut plugins = Plugins::init(|_plugin| PluginData {}).await;
        plugins.plugins.insert(AvailablePlugins::error, Box::new(crate::error::Plugin::new(PluginData {}).await));

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
        data: &serde_json::Value,
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

    pub fn get_events_overview(
        &self,
        plugin: &AvailablePlugins,
        events: &Vec<CompressedEvent>,
    ) -> Option<View> {
        self.plugins
            .get(plugin)
            .unwrap()
            .get_events_overview(events)
    }
}
