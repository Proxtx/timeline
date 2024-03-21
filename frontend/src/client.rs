use std::path::{Path, PathBuf};

use leptos::{view, IntoView};

use crate::plugin_manager::{PluginEventData, Style};

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
        let path = data.get_data::<PathBuf>()?;
        Ok(Box::new(move || {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            view! { <h1>{filename}</h1> }.into_view()
        }))
    }

    fn get_style(&self) -> crate::plugin_manager::Style {
        Style::Acc1
    }
}
