use client_api::plugin::{IconLocation, PluginTrait};
use client_api::result::EventResult;
use client_api::style::Style;
use client_api::{api, external::{leptos::{view, IntoView, View}, types::{available_plugins::AvailablePlugins, external::serde::{Deserialize, Serialize}}}};
use client_api::plugin::PluginData;
use client_api::plugin::PluginEventData;

#[derive(Serialize, Deserialize)]
struct Error {
    plugin: Option<AvailablePlugins>,
    error: String,
}

pub struct Plugin {}

impl PluginTrait for Plugin {
    fn get_style(&self) -> Style {
        Style::Light
    }
    async fn new(_data: PluginData) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }

    fn get_component(
        &self,
        data: PluginEventData,
    ) -> EventResult<Box<dyn FnOnce() -> leptos::View>> {
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
            }
            .into_view()
        }))
    }

    fn get_icon(&self) -> IconLocation {
        IconLocation::Custom(api::relative_url("/icons/errorIcon.svg").unwrap())
    }
}
