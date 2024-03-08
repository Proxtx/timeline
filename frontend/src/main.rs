use leptos::*;
use leptos_router::*;
use stylers::style;

mod wrappers;

use wrappers::{StyledView, TitleBar};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <MainView/> })
}

#[component]
fn MainView() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/timeline/:day" view=Timeline/>
            </Routes>
        </Router>
    }
}

#[component]
fn Timeline() -> impl IntoView {
    view! {
        <StyledView>
            <TitleBar title="Hello" description=Some("Whaaazzz up".to_string())/>
        </StyledView>
    }
}
