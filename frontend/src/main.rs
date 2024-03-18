use std::collections::HashMap;

use chrono::DateTime;
use leptos::*;
use leptos_router::*;

mod api;
mod timeline;
mod wrappers;

use types::{api::AvailablePlugins, timing::TimeRange};
use wrappers::{StyledView, TitleBar};

use crate::api::api_request;

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
    let clbkc = |range: TimeRange| {
        spawn_local(async move {
            let t: Result<HashMap<AvailablePlugins, >, _> = api_request("/events", &range).await;
            logging::log!("{:?}", t);
        });
    };
    let range = TimeRange {
        start: DateTime::parse_from_str(
            "2024 Jan 13 12:09:14.274 +0000",
            "%Y %b %d %H:%M:%S%.3f %z",
        )
        .unwrap()
        .into(),
        end: DateTime::parse_from_str("2024 Mar 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
            .unwrap()
            .into(),
    };

    view! {
        <StyledView>
            <TitleBar subtitle=Some("Whaaazzz up".to_string())/>
            <timeline::Timeline callback=clbkc range=range></timeline::Timeline>
        </StyledView>
    }
}
