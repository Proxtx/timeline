use std::collections::HashMap;

use leptos::{view, IntoView, View};
use types::api::AvailablePlugins;

use crate::{event_manager::EventResult, Plugins};

pub trait Plugin {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
        where 
            Self:Sized;
    fn get_component(&self, data: PluginEventData) -> EventResult<Box<dyn Fn() -> View>>;
}

pub struct PluginData {}

#[derive(Clone)]
pub struct PluginManager {
    plugins: HashMap<AvailablePlugins, std::sync::Arc<Box<dyn Plugin>>>,
}

impl PluginManager {
    pub async fn new() -> Self {
        let plugins = Plugins::init(|_plugin| PluginData {}).await;

        PluginManager {
            plugins: plugins.plugins.into_iter().map(|(k,v)| (k, std::sync::Arc::new(v))).collect(),
        }
    }

    pub fn get_component(&self, plugin: AvailablePlugins, data: String) -> EventResult<impl Fn() -> View> {
        self.plugins.get(&plugin).unwrap().get_component(PluginEventData { data })
    }
}

pub struct PluginEventData {
    data: String,

}