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
        let path = data.get_data::<PathBuf>()?.as_os_str().to_str().unwrap().to_string();
        let path_encoded = leptos::window().get("encodeURIComponent").unwrap().dyn_into::<Function>().unwrap().call1(&JsValue::null(), &JsValue::from_str(&path)).unwrap().as_string().unwrap();
        let url = api::relative_url("/api/plugin/timeline_plugin_media_scan/file/").unwrap().join(&path_encoded).unwrap().as_str().to_string();

        Ok(Box::new(move || {
            view! {
                <video style:width="100%" style:color="var(--lightColor)" src=url controls>
                    Loading video.
                </video>
            }.into_view()
        }))
    }

    fn get_style(&self) -> crate::plugin_manager::Style {
        Style::Acc1
    }
}
