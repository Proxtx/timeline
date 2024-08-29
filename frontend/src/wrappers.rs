use {chrono::Utc, leptos::wasm_bindgen::JsCast, leptos::*, stylers::style, web_sys::MouseEvent};

#[component]
pub fn TitleBar(
    #[prop(into, default=None.into())] subtitle: MaybeSignal<Option<String>>,
    #[prop(into, default=Callback::new(|_| {}))] subtitle_click_callback: Callback<MouseEvent, ()>,
) -> impl IntoView {
    let style = style! {
        .wrapper {
            width: 100%;
            display: flex;
            align-items: center;
            flex-direction: column;
            background-color: var(--darkColor);
            --padding: calc(var(--contentSpacing) * 3.5);
            padding-top: var(--padding);
            padding-bottom: var(--padding);
            gap: calc(var(--contentSpacing) * 1.5);
        }

        .titleWrapper {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            gap: var(--contentSpacing);
        }

        .logo {
            height: 40px;
            transition: 500ms;
            transform: rotate(0deg);
        }

        .subtitle {
            color: var(--accentColor1);
            text-decoration: none;
        }
    };

    view! { class=style,
        <div class="wrapper">
            <div class="titleWrapper">
                <img
                    src="/icons/logo_transparent.png"
                    class="logo"
                    on:click=|v| {
                        event_target::<web_sys::HtmlElement>(&v)
                            .style()
                            .set_property("transform", "rotate(360deg)")
                            .unwrap();
                        let _ = leptos::window().location().reload();
                    }
                />

                <h1 class="title">Timeline</h1>
            </div>
            {move || match subtitle() {
                Some(v) => {
                    view! { class=style,
                        <a href="javascript:" class="subtitle" on:click=subtitle_click_callback>
                            {v}
                        </a>
                    }
                        .into_view()
                }
                None => view! {}.into_view(),
            }}

        </div>
    }
}

#[component]
pub fn StyledView(children: Children) -> impl IntoView {
    let stylers_class = style! {
        .view {
            display: flex;
            flex-direction: column;
            width: 100%;
            height: 100%;
        }
    };
    view! { class=stylers_class, <div class="view">{children()}</div> }
}

#[component]
pub fn Login(update_authentication: WriteSignal<i64>) -> impl IntoView {
    let css = style! {
        .pwdInput {
            border: none;
            width: 100%;
            box-sizing: border-box;
            background-color: var(--accentColor2);
            padding: var(--contentSpacing);
            color: var(--lightColor);
        }
        .pwdInput::placeholder{
            color: var(--lightColor);
        }
        .pwdInput:focus{
            outline: none;
        }
    };
    view! { class=css,
        <div class="errorWrapper">
            <h3>Login</h3>
            <br/>
            <input
                class="pwdInput"
                type="password"
                placeholder="Password"
                on:change=move |e| {
                    set_password_cookie(event_target_value(&e));
                    update_authentication(Utc::now().timestamp_millis());
                }
            />

        </div>
    }
}

fn set_password_cookie(password: String) {
    let html_doc: web_sys::HtmlDocument = document().dyn_into().unwrap();
    let mut cookie = cookie::Cookie::new("pwd", password);
    cookie.set_path("/");
    html_doc
        .set_cookie(&format!(
            "{}; expires=Fri, 31 Dec 9999 23:59:59 GMT; SameSite=None; Secure",
            cookie
        ))
        .unwrap();
}
