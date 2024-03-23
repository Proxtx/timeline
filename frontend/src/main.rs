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
                <Route path="/timeline" view=Timeline/>
                <Route path="/" view=Redirect/>
            </Routes>
        </Router>
    }
}

#[component] 
fn Redirect() -> impl IntoView {
    use_navigate()("/timeline/", NavigateOptions::default());
    view! { <div class="intoWrapper">"Redirecting"</div> }
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
        .dateSelect {
            padding: var(--contentSpacing);
            background-color: var(--accentColor1);
            color: var(--lightColor);
            width: 100%;
            border: none;
            box-sizing: border-box;
            letter-spacing: 1px;
            font-family: Rubik;
            text-align: center;
        }
        .dateSelect:focus {
            outline: none;
        }

        .dateSelectWrapper {
            max-height: 0px;
            transition: 0.1s;
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

    let date_input_parser = move |c| {
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
    };

    let (last_authentication_attempt, write_last_authentication_attempt) = create_signal(Utc::now().timestamp_millis());

    let authentication = create_resource(last_authentication_attempt, |_| async move {
        api::api_request::<(), ()>("/auth", &()).await
    });

    view! {
        <StyledView>
            {move || match range() {
                Ok(range) => {
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
                            style:max-height=move || {
                                if date_select_expanded() { "100px" } else { "0px" }
                            }
                        >

                            <input
                                class="dateSelect"
                                on:change=date_input_parser
                                type="date"
                                prop:value=move || {
                                    let local = DateTime::<Local>::from(range.start);
                                    format!("{}", { local.format("%Y-%m-%d") })
                                }

                                style:color-scheme="dark"
                            />
                        </div>

                        {move || {
                            match authentication() {
                                None => ().into_view(),
                                Some(Err(e)) => {
                                    match e {
                                        APIError::AuthenticationError => {
                                            view! {
                                                <Login update_authentication=write_last_authentication_attempt/>
                                            }
                                                .into_view()
                                        }
                                        e => {
                                            view! {
                                                <div class="errorWrapper">
                                                    {move || format!("Error requesting authentication: {}", e)}
                                                </div>
                                            }
                                                .into_view()
                                        }
                                    }
                                }
                                Some(Ok(_)) => {
                                    let r3 = range.clone();
                                    let r2 = range.clone();
                                    view! { class=css,
                                        <timeline::Timeline
                                            callback=write_time_callback
                                            range=r3
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
                            }
                        }}
                    }
                        .into_view()
                }
                Err(e) => {
                    view! {
                        <TitleBar subtitle=Some("Error loading Day".to_string())/>

                        <div class="errorWrapper">
                            {move || format!("Error loading date: {}", e)}
                        </div>
                    }
                        .into_view()
                }
            }}

        </StyledView>
    }
}

#[component]
fn Login(
    update_authentication: WriteSignal<i64>
) -> impl IntoView{
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
    html_doc.set_cookie(&cookie.to_string()).unwrap();
}