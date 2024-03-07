use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <MainView/> })
}

#[component]
fn MainView() -> impl IntoView {
    view! { <TitleBar title="Hello" description=Some("Whaaazzz up".to_string())/> }
}

#[component]
fn TitleBar(
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
