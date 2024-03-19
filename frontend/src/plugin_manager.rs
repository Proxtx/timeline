use std::collections::HashMap;

use leptos::{view, IntoView, View};
use types::api::AvailablePlugins;

use crate::Plugins;

pub trait Plugin {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where Self:Sized;
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

    pub fn get_component(&self, plugin: AvailablePlugins, data: String) -> impl Fn() -> View {
        || {
            view! { <h1>Hello</h1> }.into_view()
        }
    }
}
