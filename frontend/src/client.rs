use leptos::{view, IntoView, View};
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
        let (range, url) = data.get_data::<(TimeRange, Url)>()?;
        Ok(Box::new(move || -> View {
            view! { <iframe src=move || url.to_string()>Loading iframe</iframe> }.into_view()
        }))
    }
}
