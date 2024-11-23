use client_api::external::types::available_plugins::AvailablePlugins;
use client_api::plugin::PluginData;
use client_api::plugin::PluginTrait;
use std::collections::HashMap;

#[link_proc_macro::generate_frontend_plugins]
pub struct Plugins<'a> {
    pub plugins: HashMap<AvailablePlugins, Box<dyn PluginTrait + 'a>>,
}
