//! Re-usable layout widgets. Was a stylers-scoped module; now uses plain
//! classes defined in `style.css`.

use chrono::Utc;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn StyledView(children: Children) -> impl IntoView {
    view! { <div class="view">{children()}</div> }
}

#[component]
pub fn TitleBar(
    #[prop(into, optional)] subtitle: Signal<Option<String>>,
    #[prop(into, optional)] on_subtitle_click: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let click = move |ev: web_sys::MouseEvent| {
        if let Some(cb) = on_subtitle_click.as_ref() {
            cb.run(ev);
        }
    };

    view! {
        <div class="titleBar">
            <div class="titleBarInner">
                <img
                    src="/icons/logo_transparent.png"
                    class="logo"
                    on:click=move |ev| {
                        if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
                            let _ = el.style().set_property("transform", "rotate(360deg)");
                        }
                        if let Some(win) = web_sys::window() {
                            let _ = win.location().reload();
                        }
                    }
                />
                <h1 class="title">Timeline</h1>
            </div>
            {move || {
                subtitle.get().map(|text| view! {
                    <a href="javascript:" class="subtitle" on:click=click>
                        {text}
                    </a>
                })
            }}
        </div>
    }
}

#[component]
pub fn Login(update_authentication: WriteSignal<i64>) -> impl IntoView {
    view! {
        <div class="errorWrapper">
            <h3>Login</h3>
            <br />
            <input
                class="pwdInput"
                type="password"
                placeholder="Password"
                on:change=move |e| {
                    let value = event_target_value(&e);
                    set_password_cookie(value);
                    update_authentication.set(Utc::now().timestamp_millis());
                }
            />
        </div>
    }
}

fn set_password_cookie(password: String) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(html_doc) = doc.dyn_into::<web_sys::HtmlDocument>() else {
        return;
    };
    let mut cookie = cookie::Cookie::new("pwd", password);
    cookie.set_path("/");
    let _ = html_doc.set_cookie(&format!(
        "{}; expires=Fri, 31 Dec 9999 23:59:59 GMT; SameSite=None; Secure",
        cookie
    ));
}
