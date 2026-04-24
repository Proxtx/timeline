//! Mount a Leptos view into a host element's shadow root.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::context::PluginContext;

/// Mount `render(ctx)` into the shadow root hanging off `host`. Creates
/// an open shadow root if the host doesn't have one yet. Leaks the handle
/// so the view outlives this call.
pub fn mount_plugin<F, V>(host: web_sys::HtmlElement, ctx: PluginContext, render: F)
where
    F: FnOnce(PluginContext) -> V + 'static,
    V: IntoView + 'static,
{
    let shadow = ensure_shadow_root(&host);

    let doc = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let mount_el = doc
        .create_element("div")
        .expect("create mount div")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div is HtmlElement");
    shadow
        .append_child(&mount_el)
        .expect("append mount div to shadow root");

    let handle = leptos::mount::mount_to(mount_el, move || render(ctx));
    handle.forget();
}

/// Dynamic variant: the render function returns a boxed view. Useful when
/// the plugin picks between variants at runtime.
pub fn mount_plugin_dyn<F>(host: web_sys::HtmlElement, ctx: PluginContext, render: F)
where
    F: FnOnce(PluginContext) -> AnyView + 'static,
{
    mount_plugin(host, ctx, render)
}

fn ensure_shadow_root(host: &web_sys::HtmlElement) -> web_sys::ShadowRoot {
    if let Some(existing) = host.shadow_root() {
        existing
    } else {
        let init = web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open);
        host.attach_shadow(&init).expect("attach shadow root")
    }
}
