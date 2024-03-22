use std::{ops::Deref, path::{Path, PathBuf}, str::FromStr};

use leptos::{logging, view, IntoView};
use url::Url;
use web_sys::{js_sys::{self, Function}, wasm_bindgen::JsValue};
use leptos::wasm_bindgen::JsCast;

use crate::{api, plugin_manager::{PluginEventData, Style}};

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

    fn get_component(&self, data: PluginEventData) -> crate::event_manager::EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let path = data.get_data::<PathBuf>()?;
        let extension = path.extension().unwrap().to_str().unwrap().to_lowercase().to_string();
        let path_string = path.as_os_str().to_str().unwrap().to_string();
        let path_encoded = api::encode_url_component(&path_string);
        let url = api::relative_url("/api/plugin/timeline_plugin_media_scan/file/").unwrap().join(&path_encoded).unwrap().as_str().to_string();
        Ok(Box::new(move || {
            view! {
                {match extension.as_str() {
                    "mp4" | "mkv" | "webm" | "mov" => {
                        view! {
                            <video
                                style:width="100%"
                                style:color="var(--lightColor)"
                                src=url
                                controls
                            >
                                Loading video.
                            </video>
                        }
                            .into_view()
                    }
                    _ => view! { <img style:width="100%" src=url/> }.into_view(),
                }}
            }.into_view()
        }))
    }

    fn get_style(&self) -> crate::plugin_manager::Style {
        Style::Acc1
    }
}
