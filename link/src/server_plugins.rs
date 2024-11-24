use {
    server_api::external::rocket::Route,
    server_api::external::types::available_plugins::AvailablePlugins,
    server_api::plugin::PluginData, server_api::plugin::PluginTrait, std::collections::HashMap,
};

#[link_proc_macro::generate_server_plugins]
pub struct Plugins<'a> {
    pub plugins: HashMap<AvailablePlugins, Box<dyn PluginTrait + 'a>>,
    pub routes: HashMap<AvailablePlugins, Vec<Route>>,
}
