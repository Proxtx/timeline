use chrono::{DateTime, Utc};
use leptos::{html::div, *};
use leptos_router::*;

mod api;
mod timeline;
mod wrappers;
mod plugin_manager;
mod event_manager;

use plugin_manager::{Plugin, PluginData};
use serde::Deserialize;
use types::{api::{AvailablePlugins, CompressedEvent}, timing::TimeRange};
use wrappers::{StyledView, TitleBar};

mod client;

use crate::api::api_request;

include!(concat!(env!("OUT_DIR"), "/plugins.rs"));

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
    let (read_current_time, write_current_time) = create_signal::<TimeRange>( TimeRange {
        start: DateTime::parse_from_str(
            "2024 Jan 13 12:09:14.274 +0000",
            "%Y %b %d %H:%M:%S%.3f %z",
        )
        .unwrap()
        .into(),
        end: DateTime::parse_from_str("2024 Jan 13 13:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
            .unwrap()
            .into(),
    });
    let clbkc = move |range: TimeRange| {
        write_current_time(range)
    };

    let ac = create_action(|task: &String| async {
        plugin_manager::PluginManager::new().await
    });

    ac.dispatch("Hello".to_string());

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

    let v = ac.value();

    let r2 = range.clone();

    view! {
        <StyledView>
            <TitleBar subtitle=Some("Whaaazzz up".to_string())/>
            <timeline::Timeline callback=clbkc range=range></timeline::Timeline>
            <event_manager::EventManger
                available_range=r2
                current_range=read_current_time
            ></event_manager::EventManger>
            {move || match v.get() {
                Some(v) => {
                    div()
                        .child(
                            v
                                .get_component(
                                    AvailablePlugins::timeline_plugin_media_scan,
                                    "data".to_string(),
                                ),
                        )
                        .into_view()
                }
                None => view! {}.into_view(),
            }}

        </StyledView>
    }
}
