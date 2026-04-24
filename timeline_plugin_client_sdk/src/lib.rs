//! Timeline plugin client SDK.
//!
//! A plugin's UI is a wasm bundle built with `trunk`. The main timeline
//! frontend creates a shadow root per event, then imports the plugin's JS
//! and calls the exported `__timeline_plugin_render` function (which this
//! SDK generates through [`plugin_entry!`]). Plugins write their UI in
//! Leptos exactly like any other component — the SDK just handles the
//! plumbing.

pub mod api;
pub mod context;
pub mod mount;
pub mod style;

pub use api::ApiClient;
pub use context::{PluginContext, RenderMode};
pub use mount::{mount_plugin, mount_plugin_dyn};
pub use style::Style;

pub use types::api::{APIError, APIResult, CompressedEvent};
pub use types::timing::{TimeRange, Timing};

// Re-exports that the `plugin_entry!` macro expands into.
pub use serde_wasm_bindgen;
pub use wasm_bindgen;
pub use web_sys;

/// Declare the plugin's render entrypoint. Your function receives a
/// [`PluginContext`] and returns an `impl IntoView`.
///
/// ```ignore
/// use timeline_plugin_client_sdk::*;
/// use leptos::prelude::*;
///
/// fn render(ctx: PluginContext) -> impl IntoView {
///     view! { <div>{ctx.event.title.clone()}</div> }
/// }
///
/// plugin_entry!(render);
/// ```
#[macro_export]
macro_rules! plugin_entry {
    ($func:ident) => {
        #[$crate::wasm_bindgen::prelude::wasm_bindgen]
        pub fn __timeline_plugin_render(
            shadow_host: $crate::web_sys::HtmlElement,
            payload: $crate::wasm_bindgen::JsValue,
        ) -> ::std::result::Result<(), $crate::wasm_bindgen::JsValue> {
            let ctx: $crate::PluginContext =
                $crate::serde_wasm_bindgen::from_value(payload)
                    .map_err(|e| $crate::wasm_bindgen::JsValue::from_str(&e.to_string()))?;
            $crate::mount_plugin(shadow_host, ctx, $func);
            Ok(())
        }
    };
}
