use std::str::FromStr;

use chrono::{Date, DateTime, Days, NaiveDate, SubsecRound, Timelike, Utc};
use leptos::{html::div, *};
use leptos_router::*;

mod api;
mod timeline;
mod wrappers;
mod plugin_manager;
mod event_manager;

use plugin_manager::{Plugin, PluginData};
use serde::Deserialize;
use stylers::style;
use types::{api::{APIError, APIResult, AvailablePlugins, CompressedEvent}, timing::TimeRange};
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
                <Route path="/timeline/:date" view=Timeline/>
                <Route path="/error/:error" view=ErrorRoute/>
            </Routes>
        </Router>
    }
}


#[derive(Params, PartialEq, Clone)]
struct ErrorParams {
    error: Option<String>
}

#[component]
fn ErrorRoute(
) -> impl IntoView {
    let style = style! {
        .wrapper {
            padding: var(--contentSpacing);
            background-color: var(--accentColor1Light);
            color: var(--lightColor);
        }
    };
    let error = use_params::<ErrorParams>();
    view! { class=style,
        <StyledView>
            <TitleBar subtitle=Some("Error".to_string())/>
            <div class="wrapper">{move || error.get().unwrap().error.unwrap()}</div>
        </StyledView>
    }
}

#[derive(Params, PartialEq, Clone)]
struct TimelineParams {
    date: Option<String>
}

impl TimelineParams {
    pub fn get_range (&self) -> APIResult<TimeRange> {
        let selected_day = match &self.date {
            Some(v) => {
                match DateTime::from_str(v) {
                    Ok(date) => {
                        only_date(&date)
                    }
                    Err(e) => {
                        return Err(APIError::Custom(format!("{}", e)))
                    }
                }
            }
            None => {
                only_date(&Utc::now())
            }
        };

        let next_day = selected_day.checked_add_days(Days::new(1)).unwrap();
        Ok(TimeRange { start: selected_day, end: next_day })
    }
}

fn only_date (date: &DateTime<Utc>) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(date.date_naive().and_hms_opt(0, 0, 0).unwrap(), Utc)
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

    let plugin_manager = create_action(|_: &()| async {
        plugin_manager::PluginManager::new().await
    });

    plugin_manager.dispatch(());

    let params = use_params::<TimelineParams>();

    let range = move || match params() {
        Ok(v) => {

        }
        Err(e) => {
            let navigate = use_navigate();
            navigate(&format!("/error/{}", api::encode_url_component(&format!("{}", e))), leptos_router::NavigateOptions::default());
            Err(e)
        }
    };

    let range = TimeRange {
        start: DateTime::parse_from_str(
            "2022 Jan 13 12:09:14.274 +0000",
            "%Y %b %d %H:%M:%S%.3f %z",
        )
        .unwrap()
        .into(),
        end: DateTime::parse_from_str("2024 Mar 22 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
            .unwrap()
            .into(),
    };

    let r2 = range.clone();

    view! {
        <StyledView>
            <TitleBar subtitle=Some("Whaaazzz up".to_string())/>
            <timeline::Timeline callback=clbkc range=range></timeline::Timeline>
            {move || match plugin_manager.value()() {
                Some(plg) => {
                    view! {
                        <event_manager::EventManger
                            available_range=r2.clone()
                            current_range=read_current_time
                            plugin_manager=plg
                        ></event_manager::EventManger>
                    }
                        .into_view()
                }
                None => view! { Loading Plugins }.into_view(),
            }}

        </StyledView>
    }
}
