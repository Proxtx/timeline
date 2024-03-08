use leptos::*;
use stylers::style;

#[component]
pub fn TitleBar(
    #[prop(into)] title: MaybeSignal<String>,
    #[prop(into, default=None.into())] description: MaybeSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <h1>{title}</h1>

        {move || match description() {
            Some(v) => view! { <b>{v}</b> }.into_view(),
            None => view! {}.into_view(),
        }}
    }
}

#[component]
pub fn StyledView(children: Children) -> impl IntoView {
    let stylers_class = style! {
        .one {
            background-color: red;
        }
    };
    view! { class=stylers_class, <div class="one">{children()}</div> }
}
