use {
    crate::{api::relative_url, plugin_manager}, leptos::{view, IntoView, View}, serde::{Deserialize, Serialize}, types::api::AvailablePlugins
};

#[derive(Serialize, Deserialize)]
struct Error {
    plugin: Option<AvailablePlugins>, 
    error: String
}

pub struct Plugin {}

impl plugin_manager::Plugin for Plugin {
    fn get_style(&self) -> plugin_manager::Style {
        plugin_manager::Style::Light
    }
    async fn new(_data: plugin_manager::PluginData) -> Self
        where
            Self: Sized {
        Plugin {}
    }

    fn get_component(&self, data: plugin_manager::PluginEventData) -> crate::plugin_manager::EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let data = data.get_data::<Error>()?;
        Ok(Box::new(move || -> View {
            view! {
                <h3>
                    {move || {
                        data.plugin
                            .clone()
                            .map_or("Unknown Plugin Source".to_string(), |v| { v.to_string() })
                    }}

                </h3>
                <a>{move || { data.error.clone() }}</a>
            }.into_view()
        }))
    }

    fn get_icon(&self) -> plugin_manager::IconLocation {
        plugin_manager::IconLocation::Custom(relative_url("/icons/errorIcon.svg").unwrap())
    }
}
