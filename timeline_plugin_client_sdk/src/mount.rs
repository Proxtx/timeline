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

    // Shadow DOM is style-isolated: the document's stylesheets — including the
    // plugin's own bundled `style.css` — do not cross the boundary, so without
    // this the plugin view renders structurally unstyled. (Theme CSS custom
    // properties set on a host ancestor still inherit through the boundary,
    // which is why colors work but layout doesn't.) Link the plugin's CSS,
    // served by the main server at `/plugin_web/<name>/style.css`, into the
    // shadow root. Plugins without a `style.css` get a harmless 404.
    inject_stylesheet(&doc, &shadow, &ctx.plugin_name);

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

/// Append a `<link rel="stylesheet">` to the plugin's bundled CSS into the
/// shadow root. Idempotent: skips if a link with the same href is present.
fn inject_stylesheet(doc: &web_sys::Document, shadow: &web_sys::ShadowRoot, plugin_name: &str) {
    let href = format!("/plugin_web/{}/style.css", plugin_name);
    if let Ok(Some(_)) = shadow.query_selector(&format!("link[href=\"{}\"]", href)) {
        return;
    }
    if let Ok(link) = doc.create_element("link") {
        let _ = link.set_attribute("rel", "stylesheet");
        let _ = link.set_attribute("href", &href);
        let _ = shadow.append_child(&link);
    }
}

fn ensure_shadow_root(host: &web_sys::HtmlElement) -> web_sys::ShadowRoot {
    if let Some(existing) = host.shadow_root() {
        existing
    } else {
        let init = web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open);
        host.attach_shadow(&init).expect("attach shadow root")
    }
}
