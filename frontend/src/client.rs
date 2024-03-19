use leptos::{view, IntoView};

use crate::plugin_manager::PluginEventData;

pub struct Plugin {}

impl crate::Plugin for Plugin {
    async fn new(
        _data: crate::plugin_manager::PluginData,
    ) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }

    fn get_component(&self, data: PluginEventData) -> crate::event_manager::EventResult<Box<dyn Fn() -> leptos::View>> {
        Ok(Box::new(|| {
            view! { <h1>Hello</h1> }.into_view()
        }))
    }
}
