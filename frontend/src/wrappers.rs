use leptos::*;
use stylers::style;

#[component]
pub fn TitleBar(
    #[prop(into, default=None.into())] subtitle: MaybeSignal<Option<String>>,
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
        }

        .subtitle {
            color: var(--accentColor1);
        }
    };

    view! { class=style,
        <div class="wrapper">
            <div class="titleWrapper">
                <img src="/icons/logo_transparent.png" class="logo"/>
                <h1 class="title">Timeline</h1>
            </div>
            {move || match subtitle() {
                Some(v) => view! { class=style, <a class="subtitle">{v}</a> }.into_view(),
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
