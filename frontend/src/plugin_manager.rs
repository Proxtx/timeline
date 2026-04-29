//! Fetch the server's plugin manifest and mount plugin UIs into shadow roots.
//!
//! Plugins ship their Leptos code as a trunk-built bundle under
//! `/plugin_web/<name>/` on the main server. Each bundle's JS module
//! exports a `__timeline_plugin_render(host: HTMLElement, ctx: any)` function
//! that the SDK's `plugin_entry!` macro generates.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use types::api::CompressedEvent;

use crate::api::api_request;
use crate::style::Style;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub icon: Option<String>,
    /// Relative URL inside `/plugin_web/<name>/` of the JS entrypoint.
    /// Unset → no UI; events render title only.
    #[serde(default)]
    pub web_entry: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Event,
    Overview,
    Standalone,
}

#[derive(Debug, Serialize, Clone)]
pub struct PluginContext {
    pub plugin_name: String,
    pub api_base: String,
    pub event: CompressedEvent,
    pub style: Style,
    pub mode: RenderMode,
}

#[derive(Debug, Clone, Default)]
pub struct PluginManager {
    plugins: HashMap<String, PluginManifest>,
}

impl PluginManager {
    pub async fn load() -> Self {
        match api_request::<Vec<PluginManifest>, ()>("/plugins", &()).await {
            Ok(v) => {
                let plugins = v.into_iter().map(|m| (m.name.clone(), m)).collect();
                PluginManager { plugins }
            }
            Err(_) => PluginManager::default(),
        }
    }

    pub fn names(&self) -> Vec<String> {
        let mut v: Vec<_> = self.plugins.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn get(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name)
    }

    pub fn style(&self, name: &str) -> Style {
        self.plugins
            .get(name)
            .map(|m| m.style.clone())
            .unwrap_or_default()
    }

    pub fn display_name(&self, name: &str) -> String {
        self.plugins
            .get(name)
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| name.to_string())
    }

    /// Resolve a plugin's icon URL. Three branches:
    /// 1. manifest's `icon` is an absolute URL (`/...` or `http...`) → use as-is.
    /// 2. manifest's `icon` is a relative path → served from the plugin's
    ///    trunk bundle at `/plugin_web/<name>/<icon>`.
    /// 3. manifest's `icon` is `None` → convention is `icon.svg` inside the
    ///    plugin's bundle (plugin clients ship `../icon.svg` via trunk
    ///    `copy-file`). Missing files chain through `icon.png` and finally
    ///    fall back to the frontend's generic `/icons/event.svg` via
    ///    `<img onerror>` in `events_display`.
    pub fn icon_url(&self, name: &str) -> String {
        if let Some(m) = self.plugins.get(name) {
            if let Some(icon) = &m.icon {
                if icon.starts_with('/') || icon.starts_with("http") {
                    return icon.clone();
                }
                return format!("/plugin_web/{}/{}", name, icon.trim_start_matches('/'));
            }
        }
        format!("/plugin_web/{}/icon.svg", name)
    }

    /// Mount a plugin UI into the shadow root of `host`. If the plugin has
    /// no `web_entry`, or the dynamic import fails, a fallback title is
    /// shown instead.
    pub async fn mount(&self, plugin_name: &str, host: web_sys::HtmlElement, event: CompressedEvent) {
        let manifest = match self.plugins.get(plugin_name).cloned() {
            Some(m) => m,
            None => {
                render_fallback(&host, &format!("Unknown plugin: {}", plugin_name));
                return;
            }
        };

        let Some(entry) = manifest.web_entry.clone() else {
            render_fallback(&host, &event.title);
            return;
        };

        let url = format!(
            "/plugin_web/{}/{}",
            manifest.name,
            entry.trim_start_matches('/')
        );

        let ctx = PluginContext {
            plugin_name: manifest.name.clone(),
            api_base: format!("/api/plugin/{}", manifest.name),
            event: event.clone(),
            style: manifest.style.clone(),
            mode: RenderMode::Event,
        };

        match dynamic_import_render(&url, &host, &ctx).await {
            Ok(()) => {}
            Err(e) => {
                leptos::logging::warn!(
                    "plugin {} mount failed: {:?} (falling back to title only)",
                    manifest.name,
                    e
                );
                render_fallback(&host, &event.title);
            }
        }
    }
}

/// Dynamic `import()` of the plugin module, then call the exported
/// `__timeline_plugin_render(host, ctx)`.
async fn dynamic_import_render(
    url: &str,
    host: &web_sys::HtmlElement,
    ctx: &PluginContext,
) -> Result<(), JsValue> {
    // Modules built by trunk in default `--target web` mode export a
    // default `init` function that loads the wasm asynchronously. We
    // must call it before any other named export becomes usable.
    let importer = js_sys::Function::new_with_args("url", "return import(url);");
    let promise = importer
        .call1(&JsValue::NULL, &JsValue::from_str(url))?
        .dyn_into::<js_sys::Promise>()?;
    let module = wasm_bindgen_futures::JsFuture::from(promise).await?;

    // Initialize the plugin's wasm. `module.default()` returns a Promise.
    let init_fn = js_sys::Reflect::get(&module, &JsValue::from_str("default"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("plugin module missing default export (init)"))?;
    let init_promise = init_fn.call0(&JsValue::NULL)?;
    if let Ok(p) = init_promise.dyn_into::<js_sys::Promise>() {
        wasm_bindgen_futures::JsFuture::from(p).await?;
    }

    let render_fn = js_sys::Reflect::get(&module, &JsValue::from_str("__timeline_plugin_render"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("plugin module missing __timeline_plugin_render"))?;

    // `Timing` carries i64 nanoseconds which overflow JS Number precision
    // (2^53). Serialize i64/u64 as BigInt; the plugin's deserializer
    // (serde-wasm-bindgen::from_value) handles BigInt natively.
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_large_number_types_as_bigints(true);
    let ctx_value = ctx
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    render_fn.call2(&JsValue::NULL, host, &ctx_value)?;
    Ok(())
}

fn render_fallback(host: &web_sys::HtmlElement, title: &str) {
    host.set_inner_text(title);
}
