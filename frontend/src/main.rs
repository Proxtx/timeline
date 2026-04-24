mod api;
mod event_manager;
mod events_display;
mod plugin_manager;
mod style;
mod timeline_view;
mod wrappers;

use std::str::FromStr;

use chrono::{DateTime, Days, Local, NaiveDate, NaiveTime, TimeDelta, Utc};
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use leptos_router::{path, NavigateOptions};
use types::api::{APIError, CompressedEvent};
use types::timing::TimeRange;

use crate::api::{api_request, TimelineHostname};
use crate::event_manager::EventManager;
use crate::events_display::{DisplayWithDay, EventsViewer};
use crate::plugin_manager::PluginManager;
use crate::timeline_view::TimelineBar;
use crate::wrappers::{Login, StyledView, TitleBar};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <MainView /> });
}

#[component]
fn MainView() -> impl IntoView {
    let origin = web_sys::window()
        .map(|w| w.origin())
        .unwrap_or_default();
    provide_context(TimelineHostname(origin));

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/timeline/:date") view=Timeline />
                <Route path=path!("/timeline") view=Timeline />
                <Route path=path!("/event/latest/exclude/:exclude") view=LatestEvent />
                <Route path=path!("/event/latest") view=LatestEvent />
                <Route path=path!("/") view=Redirect />
            </Routes>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <StyledView>
            <TitleBar subtitle=Signal::derive(|| Some("404 — Not Found".to_string())) />
            <div class="errorWrapper">Was unable to find the page you are looking for.</div>
        </StyledView>
    }
}

#[component]
fn Redirect() -> impl IntoView {
    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate("/timeline", NavigateOptions::default());
    });
    view! { <div class="infoWrapper">Redirecting</div> }
}

// ---------------- /timeline[/:date] ----------------

#[derive(Params, PartialEq, Clone, Debug)]
struct TimelineParams {
    date: Option<String>,
}

fn date_from_param(date: &Option<String>) -> Result<DateTime<Utc>, APIError> {
    let source = match date {
        None => Utc::now(),
        Some(v) if v.is_empty() => Utc::now(),
        Some(v) => DateTime::<Utc>::from_str(v).map_err(|e| APIError::Custom(e.to_string()))?,
    };
    Ok(only_date_local(source))
}

fn only_date_local(d: DateTime<Utc>) -> DateTime<Utc> {
    DateTime::<Utc>::from(
        DateTime::<Local>::from(d)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap(),
    )
}

#[component]
fn Timeline() -> impl IntoView {
    let params = use_params::<TimelineParams>();

    let day_range = Memo::new(move |_| -> Result<TimeRange, APIError> {
        let p = params
            .get()
            .map_err(|e| APIError::Custom(e.to_string()))?;
        let start = date_from_param(&p.date)?;
        let end = start
            .checked_add_days(Days::new(1))
            .ok_or_else(|| APIError::Custom("invalid day overflow".into()))?;
        Ok(TimeRange { start, end })
    });

    let current_range = RwSignal::new(TimeRange {
        start: DateTime::from_timestamp_millis(0).unwrap_or_default(),
        end: DateTime::from_timestamp_millis(0).unwrap_or_default(),
    });

    Effect::new(move |_| {
        if let Ok(r) = day_range.get() {
            current_range.set(TimeRange {
                start: r.start,
                end: r.start + TimeDelta::try_hours(1).unwrap_or_default(),
            });
        }
    });

    let plugin_manager_action = Action::new_local(|_: &()| async { PluginManager::load().await });
    Effect::new(move |_| {
        if plugin_manager_action.value().get_untracked().is_none() {
            plugin_manager_action.dispatch(());
        }
    });
    let plugin_manager = Signal::derive(move || {
        plugin_manager_action.value().get().unwrap_or_default()
    });

    let last_auth = RwSignal::new(Utc::now().timestamp_millis());
    let authentication = LocalResource::new(move || {
        let _ = last_auth.get();
        async { api_request::<(), _>("/auth", &()).await }
    });

    let date_select_expanded = RwSignal::new(false);

    view! {
        <StyledView>
            {move || match day_range.get() {
                Ok(day) => {
                    let on_range_pick = Callback::new(move |r: TimeRange| current_range.set(r));
                    let day_for_subtitle = day.clone();
                    let day_for_input = day.clone();
                    let day_for_bar = day.clone();
                    let day_for_manager = day.clone();
                    view! {
                        <TitleBar
                            subtitle=Signal::derive(move || Some(
                                DateTime::<Local>::from(day_for_subtitle.start).format("%d.%m.%Y").to_string()
                            ))
                            on_subtitle_click=Callback::new(move |_| {
                                date_select_expanded.update(|v| *v = !*v);
                            })
                        />
                        <div
                            class="dateSelectWrapper"
                            style:max-height=move || if date_select_expanded.get() { "100px" } else { "0px" }
                        >
                            <input
                                class="dateSelect"
                                type="date"
                                prop:value=format!("{}", DateTime::<Local>::from(day_for_input.start).format("%Y-%m-%d"))
                                on:change=move |e| {
                                    date_select_expanded.set(false);
                                    handle_date_change(&e);
                                }
                                style:color-scheme="dark"
                            />
                        </div>
                        {move || match authentication.get() {
                            None => view! { <div class="infoWrapper">Loading...</div> }.into_any(),
                            Some(Err(APIError::AuthenticationError)) => view! {
                                <Login update_authentication=last_auth.write_only() />
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <div class="errorWrapper">{format!("Authentication error: {}", e)}</div>
                            }.into_any(),
                            Some(Ok(_)) => {
                                let day_bar = day_for_bar.clone();
                                let day_mgr = day_for_manager.clone();
                                view! {
                                    <TimelineBar
                                        range=Signal::derive(move || day_bar.clone())
                                        on_range_pick=on_range_pick
                                    />
                                    <EventManager
                                        available_range=Signal::derive(move || day_mgr.clone())
                                        current_range=Signal::derive(move || current_range.get())
                                        plugin_manager=plugin_manager
                                    />
                                }.into_any()
                            }
                        }}
                    }.into_any()
                }
                Err(e) => view! {
                    <TitleBar subtitle=Signal::derive(|| Some("Error loading Day".to_string())) />
                    <div class="errorWrapper">{format!("Error loading date: {}", e)}</div>
                }.into_any()
            }}
        </StyledView>
    }
}

fn handle_date_change(e: &web_sys::Event) {
    let value = event_target_value(e);
    let parts: Vec<i32> = value
        .split('-')
        .filter_map(|v| v.parse().ok())
        .collect();
    if parts.len() != 3 {
        return;
    }
    let Some(date) = NaiveDate::from_ymd_opt(parts[0], parts[1] as u32, parts[2] as u32) else {
        return;
    };
    let Some(naive_time) = NaiveTime::from_hms_opt(0, 0, 0) else {
        return;
    };
    let local = date.and_time(naive_time).and_local_timezone(Local).earliest();
    let Some(local) = local else { return };
    let utc: DateTime<Utc> = DateTime::from(local);
    let navigate = use_navigate();
    navigate(
        &format!("/timeline/{}", utc.to_rfc3339()),
        NavigateOptions::default(),
    );
}

// ---------------- /event/latest[/exclude/:exclude] ----------------

#[derive(Params, PartialEq, Clone, Debug)]
struct LatestParams {
    exclude: Option<String>,
}

#[component]
fn LatestEvent() -> impl IntoView {
    provide_context(DisplayWithDay(true));

    let last_auth = RwSignal::new(Utc::now().timestamp_millis());

    let range = RwSignal::new(TimeRange {
        end: Utc::now(),
        start: Utc::now() - TimeDelta::try_hours(1).unwrap_or_default(),
    });

    // Refresh the clock every time we re-authenticate, keeps "latest" fresh.
    Effect::new(move |_| {
        let _ = last_auth.get();
        range.set(TimeRange {
            end: Utc::now(),
            start: Utc::now() - TimeDelta::try_hours(1).unwrap_or_default(),
        });
    });

    let events_resource = LocalResource::new(move || {
        let r = range.get();
        async move {
            api_request::<std::collections::HashMap<String, Vec<CompressedEvent>>, _>("/events", &r).await
        }
    });

    let params = use_params::<LatestParams>();
    let exclude: Memo<Vec<String>> = Memo::new(move |_| match params.get() {
        Ok(p) => p
            .exclude
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect(),
        Err(_) => vec![],
    });

    let plugin_manager_action = Action::new_local(|_: &()| async { PluginManager::load().await });
    Effect::new(move |_| {
        if plugin_manager_action.value().get_untracked().is_none() {
            plugin_manager_action.dispatch(());
        }
    });
    let plugin_manager = Signal::derive(move || {
        plugin_manager_action.value().get().unwrap_or_default()
    });

    view! {
        <StyledView>
            {move || match events_resource.get() {
                Some(Ok(events)) => {
                    let excluded = exclude.get();
                    let events_map: std::collections::HashMap<String, Vec<CompressedEvent>> = events
                        .into_iter()
                        .filter(|(plugin, _)| !excluded.contains(plugin))
                        .collect();
                    view! {
                        <EventsViewer
                            events=Signal::derive(move || events_map.clone())
                            plugin_manager=plugin_manager
                        />
                    }.into_any()
                }
                Some(Err(APIError::AuthenticationError)) => view! {
                    <Login update_authentication=last_auth.write_only() />
                }.into_any(),
                Some(Err(e)) => view! {
                    <div class="errorWrapper">{format!("Error requesting events: {}", e)}</div>
                }.into_any(),
                None => view! { <div class="infoWrapper">Loading...</div> }.into_any(),
            }}
        </StyledView>
    }
}
