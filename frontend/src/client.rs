use leptos::{view, IntoView, View};
use stylers::style;
use types::timing::TimeRange;
use url::Url;

use crate::plugin_manager;

pub struct Plugin {}

impl plugin_manager::Plugin for Plugin {
    fn get_style(&self) -> plugin_manager::Style {
        plugin_manager::Style::Acc2
    }
    async fn new(_data: plugin_manager::PluginData) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }
    fn get_component(
        &self,
        data: plugin_manager::PluginEventData,
    ) -> crate::event_manager::EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let (range, mut url) = data.get_data::<(TimeRange, Url)>()?;
        url.set_path("/observe/");
        url.set_query(Some(&format!("skipWelcome=true&start={}&end={}", range.start.timestamp_millis(), range.end.timestamp_millis())));

        Ok(Box::new(move || -> View {
            view! {<iframe style:height = "250px" style:width = "100%" style:border = "none" class="wrapper" src=move || url.to_string()>Loading iframe</iframe> }.into_view()
        }))
    }
}
