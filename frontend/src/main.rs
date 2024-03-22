use std::{ops::Deref, str::FromStr};

use chrono::{Date, DateTime, Days, Local, NaiveDate, NaiveTime, SubsecRound, Timelike, Utc};
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
use web_sys::{js_sys::{Function, JsString, Reflect}, wasm_bindgen::{JsCast, JsValue}};
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
            </Routes>
        </Router>
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
                if v.is_empty() {
                    only_date_local(Utc::now())
                }
                else {
                    match DateTime::from_str(v) {
                        Ok(date) => {
                            only_date_local(date)
                        }
                        Err(e) => {
                            return Err(APIError::Custom(format!("{}", e)))
                        }
                    }
                }
            }
            None => {
                only_date_local(Utc::now())
            }
        };

        let next_day = selected_day.checked_add_days(Days::new(1)).unwrap();
        Ok(TimeRange { start: selected_day, end: next_day })
    }
}

fn only_date_local (date: DateTime<Utc>) -> DateTime<Utc> {
    DateTime::<Utc>::from(DateTime::<Local>::from(date).date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap())
}

#[component]
fn Timeline() -> impl IntoView {
    let css = style! {
        .dateSelectWrapper {
            background-color: var(--darkColor);
            padding: var(--contentSpacing);
            box-sizing: border-box;
        }
        .dateSelectWrapper input {
            padding: var(--contentSpacing);
            background-color: var(--accentColor1);
            color: var(--lightColor);
            width: 100%;
            border: none;
            box-sizing: border-box;
        }
        .dateSelectWrapper input:focus {
            outline: none;
        }
    };

    let (read_current_time, write_current_time) = create_signal::<TimeRange>( TimeRange {
        start: DateTime::from_timestamp_millis(0).unwrap(),
        end: DateTime::from_timestamp_millis(0).unwrap(),
    });
    let write_time_callback = move |range: TimeRange| {
        write_current_time(range)
    };

    let plugin_manager = create_action(|_: &()| async {
        plugin_manager::PluginManager::new().await
    });
    plugin_manager.dispatch(());

    let params = use_params::<TimelineParams>();
    let range = create_memo(move |_| match params() {
        Ok(v) => {
            v.get_range()
        }
        Err(e) => {
            Err(APIError::Custom(format!("{}", e)))
        }
    });

    let (date_select_expanded, write_date_select_expanded) = create_signal(false);

    view! {
        <StyledView>
            {move || match range() {
                Ok(range) => {
                    let r2 = range.clone();
                    let r3 = range.clone();
                    view! { class=css,
                        <TitleBar
                            subtitle=Signal::derive(move || {
                                Some(
                                    format!(
                                        "{}",
                                        DateTime::<Local>::from(r3.start).format("%d.%m.%Y"),
                                    ),
                                )
                            })

                            subtitle_click_callback=Callback::new(move |_| {
                                write_date_select_expanded.set(!date_select_expanded())
                            })
                        />

                        <div
                            class="dateSelectWrapper"
                            style:display=move || {
                                if date_select_expanded() { "block" } else { "none" }
                            }

                            style:color-scheme="dark"
                        >
                            <input
                                on:change=move |c| {
                                    write_date_select_expanded(false);
                                    let value = event_target_value(&c);
                                    let date: Vec<_> = value
                                        .split('-')
                                        .map(|v| { v.parse::<u32>().unwrap() })
                                        .collect();
                                    let local_date: DateTime<Local> = NaiveDate::from_ymd_opt(
                                            date[0] as i32,
                                            date[1],
                                            date[2],
                                        )
                                        .unwrap()
                                        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                                        .and_local_timezone(Local)
                                        .unwrap();
                                    let utc_date: DateTime<Utc> = DateTime::from(local_date);
                                    let navigate = leptos_router::use_navigate();
                                    navigate(
                                        &format!("/timeline/{}", &utc_date.to_rfc3339()),
                                        Default::default(),
                                    );
                                }

                                type="date"
                            />
                        </div>
                        <timeline::Timeline
                            callback=write_time_callback
                            range=range
                        ></timeline::Timeline>
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
                    }
                        .into_view()
                }
                Err(e) => {
                    view! {
                        <TitleBar subtitle=Some("Error loading Day".to_string())/>

                        <div class="errorWrapper">{move || format!("{}", e)}</div>
                    }
                        .into_view()
                }
            }}

        </StyledView>
    }
}
