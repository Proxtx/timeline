#![feature(let_chains)]

mod error;
mod event_manager;
mod events_display;
mod plugin_manager;
mod timeline;
mod wrappers;

use {
    client_api::api::api_request,
    client_api::api,
    client_api::external::types::external::chrono,
    client_api::external::types::external::chrono::{DateTime, Days, Local, NaiveDate, NaiveTime, TimeDelta, Utc},
    events_display::{DefaultEventsViewerType, EventDisplay},
    leptos::*,
    leptos_router::*,
    std::{collections::HashMap, str::FromStr},
    stylers::style,
    client_api::external::types::{
        api::{APIError, APIResult, CompressedEvent, TimelineHostname},
        timing::TimeRange,
        available_plugins::AvailablePlugins
    },
    wrappers::{Login, StyledView, TitleBar},
};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <MainView /> })
}

#[component]
fn MainView() -> impl IntoView {
    provide_context(TimelineHostname(leptos::window().origin()));

    view! {
        <Router>
            <Routes>
                <Route path="/timeline/:date" view=Timeline />
                <Route path="/timeline" view=Timeline />
                <Route path="/" view=Redirect />
                <Route path="*not_found" view=NotFound />
                <Route path="/event/latest/exclude/:exclude" view=LatestEvent />
                <Route path="/event/latest" view=LatestEvent />
            </Routes>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <StyledView>
            <TitleBar subtitle=Some("404 - Not Found".to_string()) />
            <div class="errorWrapper">Was unable to find the page you are looking for.</div>
        </StyledView>
    }
}

#[component]
fn Redirect() -> impl IntoView {
    use_navigate()("/timeline/", NavigateOptions::default());
    view! { <div class="intoWrapper">"Redirecting"</div> }
}

#[derive(Params, PartialEq, Clone)]
struct LatestParams {
    exclude: Option<String>,
}

#[component]
fn LatestEvent() -> impl IntoView {
    let style = style! {
        .wrapper {
            flex: 1 0;
            transition: 0.2s;
            overflow: auto;
        }
    };

    let (range, _write_range) = create_signal(TimeRange {
        end: Utc::now(),
        start: Utc::now()
            .checked_sub_signed(chrono::TimeDelta::try_hours(1).unwrap())
            .unwrap(),
    });
    let (last_authentication_attempt, write_last_authentication_attempt) =
        create_signal(Utc::now().timestamp_millis());
    let events = create_resource(
        move || (range(), last_authentication_attempt()),
        |(range, _)| async move {
            api_request::<HashMap<AvailablePlugins, Vec<CompressedEvent>>, _>("/events", &range)
                .await
        },
    );

    let plugin_manager =
        create_action(|_: &()| async { plugin_manager::PluginManager::new().await });
    plugin_manager.dispatch(());

    let params = use_params::<LatestParams>();
    let exclude = create_memo(move |_| match params() {
        Ok(v) => v
            .exclude
            .unwrap_or_default()
            .split(",")
            .map(|v| v.into())
            .collect::<Vec<String>>(),
        Err(_e) => vec![],
    });

    let current_event = create_memo(move |_| match events() {
        Some(v) => match v {
            Ok(v) => {
                let mut newest_event: Option<(AvailablePlugins, CompressedEvent)> = None;
                for (plugin, events) in v.into_iter() {
                    if exclude().contains(&plugin.to_string()) {
                        continue;
                    }
                    for event in events {
                        let replace = if let Some(v) = &newest_event {
                            v.1.time.cmp(&event.time).is_lt()
                        } else {
                            true
                        };
                        if replace {
                            newest_event = Some((plugin.clone(), event));
                        }
                    }
                }
                Some(Ok(newest_event))
            }
            Err(e) => Some(Err(e)),
        },
        None => None,
    });

    view! {
        <StyledView>
            {move || match (current_event(), plugin_manager.value()()) {
                (Some(current_event), Some(plugin_manager)) => {
                    match current_event {
                        Ok(v) => {
                            match v {
                                Some((plugin, event)) => {
                                    let color = format!("{}", plugin_manager.get_style(&plugin));
                                    view! { class=style,
                                        <div class="wrapper" style:background-color=color>

                                            {
                                                view! {
                                                    <EventDisplay<
                                                    CompressedEvent,
                                                    DefaultEventsViewerType,
                                                >
                                                        event=event
                                                        plugin_manager=plugin_manager
                                                        plugin=plugin
                                                        expanded=create_rw_signal(true)
                                                        slide_over=None
                                                    />
                                                }
                                            }

                                        </div>
                                    }
                                        .into_view()
                                }
                                None => {
                                    view! { <div class="infoWrapper">No Event Found</div> }
                                        .into_view()
                                }
                            }
                        }
                        Err(e) => {
                            match e {
                                APIError::AuthenticationError => {
                                    view! {
                                        <Login update_authentication=write_last_authentication_attempt />
                                    }
                                        .into_view()
                                }
                                e => {
                                    view! {
                                        <div class="errorWrapper">
                                            {move || format!("Error requesting event: {}", e)}
                                        </div>
                                    }
                                        .into_view()
                                }
                            }
                        }
                    }
                }
                _ => view! { <div class="infoWrapper">Loading</div> }.into_view(),
            }}

        </StyledView>
    }
}

#[derive(Params, PartialEq, Clone)]
struct TimelineParams {
    date: Option<String>,
}

impl TimelineParams {
    pub fn get_range(&self) -> APIResult<TimeRange> {
        let selected_day = match &self.date {
            Some(v) => {
                if v.is_empty() {
                    only_date_local(Utc::now())
                } else {
                    match DateTime::from_str(v) {
                        Ok(date) => only_date_local(date),
                        Err(e) => return Err(APIError::Custom(format!("{}", e))),
                    }
                }
            }
            None => only_date_local(Utc::now()),
        };

        let next_day = selected_day.checked_add_days(Days::new(1)).unwrap();
        Ok(TimeRange {
            start: selected_day,
            end: next_day,
        })
    }
}

fn only_date_local(date: DateTime<Utc>) -> DateTime<Utc> {
    DateTime::<Utc>::from(
        DateTime::<Local>::from(date)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap(),
    )
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
            overflow: hidden;
        }
    };

    let (read_current_time, write_current_time) = create_signal::<TimeRange>(TimeRange {
        start: DateTime::from_timestamp_millis(0).unwrap(),
        end: DateTime::from_timestamp_millis(0).unwrap(),
    });
    let write_time_callback = move |range: TimeRange| write_current_time(range);

    let plugin_manager =
        create_action(|_: &()| async { plugin_manager::PluginManager::new().await });
    plugin_manager.dispatch(());

    let params = use_params::<TimelineParams>();
    let range = create_memo(move |_| match params() {
        Ok(v) => v.get_range(),
        Err(e) => Err(APIError::Custom(format!("{}", e))),
    });

    let (date_select_expanded, write_date_select_expanded) = create_signal(false);

    let date_input_parser = move |c| {
        write_date_select_expanded(false);
        let value = event_target_value(&c);
        let date: Vec<_> = value
            .split('-')
            .map(|v| v.parse::<u32>().unwrap())
            .collect();
        let local_date: DateTime<Local> = NaiveDate::from_ymd_opt(date[0] as i32, date[1], date[2])
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

    let (last_authentication_attempt, write_last_authentication_attempt) =
        create_signal(Utc::now().timestamp_millis());

    let authentication = create_resource(last_authentication_attempt, |_| async move {
        api::api_request::<(), ()>("/auth", &()).await
    });

    view! {
        <StyledView>
            {move || match range() {
                Ok(range) => {
                    write_current_time(TimeRange {
                        start: range.start,
                        end: range
                            .start
                            .checked_add_signed(TimeDelta::try_hours(1).unwrap())
                            .unwrap(),
                    });
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
                                                <Login update_authentication=write_last_authentication_attempt />
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
                                                    <event_manager::EventManager
                                                        available_range=r2.clone()
                                                        current_range=read_current_time
                                                        plugin_manager=plg
                                                    ></event_manager::EventManager>
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
                        <TitleBar subtitle=Some("Error loading Day".to_string()) />

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
