use std::{collections::HashMap, fmt, rc::Rc, str::FromStr};

use leptos::*;
use stylers::style;
use types::{
    api::{APIResult, AvailablePlugins, CompressedEvent},
    timing::TimeRange,
};
use url::Url;

use crate::{api::{self, api_request}, plugin_manager::{self, PluginManager, Style}};

#[component]
pub fn EventManger(
    #[prop(into)] available_range: MaybeSignal<TimeRange>,
    #[prop(into)] current_range: MaybeSignal<TimeRange>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>
) -> impl IntoView {
    let available_events = create_resource(available_range, |range| async move {
        logging::log!("reloading all events");
        api_request::<HashMap<AvailablePlugins, Vec<CompressedEvent>>, _>("/events", &range).await
    });

    let current_events = create_memo(move |_: Option<&APIResult<_>>| {
        match available_events.get() {
            Some(available_events) => {
                let available_events = available_events?;
                Ok(Some(
                    available_events.into_iter()
                    .map(|(plugin, events)| {
                        (plugin, events.into_iter()
                        .filter(|current_event| current_range()
                        .overlap_timing(&current_event.time))
                        .collect::<Vec<CompressedEvent>>())
                    })
                    .filter(|(_plugin, data)| !data.is_empty())
                    .collect::<HashMap<AvailablePlugins, Vec<CompressedEvent>>>()
                ))
            }
            None => {
                Ok(None)
            }
        }
    });

    let plugin_manager_e = plugin_manager.clone();

    let currently_available_plugins = move || match current_events()? {
        Some(v) => {
            APIResult::Ok(Some(v.keys().cloned().collect::<Vec<AvailablePlugins>>()))
        }
        None => {
            Ok(None)
        }
    };

    let current_app: RwSignal<Option<AvailablePlugins>> = create_rw_signal(None);

    let selected_events = create_memo(move |_| {
        match (current_app(), current_events()) {
            (Some(app), Ok(Some(events))) => {
                Ok(Some(events[&app].clone()))
            },
            (None, Ok(Some(_))) => Ok(Some(Vec::new())),
            (_, Ok(None)) => Ok(None),
            (_, Err(e)) => Err(e)
        }
    });

    let plugin_manager_c = plugin_manager.clone();

    let current_style = move || {
        match current_app() {
            Some(v) => {
                plugin_manager_c().get_style(&v)
            },
            None => {
                Style::Acc2
            }
        }
    };



    view! {
        {move || {
            match currently_available_plugins() {
                Ok(v) => {
                    match v {
                        Some(v) => {
                            view! {
                                <AppSelect
                                    selectable_apps=v
                                    current_app=current_app
                                    plugin_manager=plugin_manager.clone()
                                />
                            }
                                .into_view()
                        }
                        None => view! { Loading }.into_view(),
                    }
                }
                Err(e) => {
                    view! { {move || format!("Error loading app selector: {}", e)} }.into_view()
                }
            }
        }}

        {move || match selected_events() {
            Ok(v) => {
                match v {
                    Some(v) => {
                        view! {
                            <EventsDisplay
                                style=Signal::derive(current_style.clone())
                                selected_events=v
                                plugin_manager=plugin_manager_e.clone()
                            />
                        }
                    }
                    None => view! { Loading }.into_view(),
                }
            }
            Err(e) => view! { {move || format!("Error loading event display: {}", e)} }.into_view(),
        }}
    }
}

#[component]
fn AppSelect (
#[prop(into)] selectable_apps: MaybeSignal<Vec<AvailablePlugins>>, 
#[prop(into)] current_app: RwSignal<Option<AvailablePlugins>>, 
#[prop(into)] plugin_manager: MaybeSignal<PluginManager>) -> impl IntoView {
    let style = style! {
        .selector {
            --padding: calc(var(--contentSpacing) * 1.5);
            height: calc(50px + 2 * var(--padding));
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: var(--padding);
            background-color: var(--darkColor);
            box-sizing: border-box;
            overflow: hidden;
        }

        .icon {
            width: 50px;
            height: 50px;
            z-index: 1;
            position: relative;
        }

        .indicator {
            background-color: red;
            width: 5px;
            position: absolute;
            left: 50%;
            top: 50%;
            height: 0%;
            transition: 0.2s;
            transform: translateX(-50%);
        }

        .iconWrap {
            position: relative;
            height: 100%;
        }
    };
    
    view! { class=style,
        <div class="selector">
            <For
                each=selectable_apps

                key=|app| format!("{}", app)

                children=move |t| {
                    let url = api::relative_url("/api/icon/")
                        .unwrap()
                        .join(&format!("{}", t))
                        .unwrap();
                    let type_2 = t.clone();
                    let type_3 = t.clone();
                    let plg = plugin_manager.clone();
                    view! { class=style,
                        <div class="iconWrap">
                            <img
                                src=url.to_string()
                                class="icon"
                                on:click=move |_| {
                                    current_app.set(Some(type_2.clone()));
                                }
                            />

                            <div
                                class="indicator"
                                style:height=move || {
                                    match current_app.get() {
                                        Some(v) => if v == t { "100%" } else { "0" }
                                        None => "0",
                                    }
                                }

                                style:background-color=move || {
                                    let style = plg().get_style(&type_3);
                                    format!("{}", style)
                                }
                            >
                            </div>
                        </div>
                    }
                }
            />

        </div>
    }
}

#[component]
fn EventsDisplay(
#[prop(into)] style: MaybeSignal<Style>,
#[prop(into)] selected_events: MaybeSignal<Vec<CompressedEvent>>,
#[prop(into)] plugin_manager: MaybeSignal<PluginManager>
) -> impl IntoView{
    let css = style! {
        .wrapper {
            flex: 1 0 auto;
            transition: 0.2s;
        }
    };

    view! { class=css,
        <div class="wrapper" style:background-color=move || { format!("{}", style.get()) }></div>
    }
}

#[component]
fn EventDisplay (
#[prop(into)] event: MaybeSignal<CompressedEvent>,
#[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
#[prop(into)] style: MaybeSignal<Style>
) -> impl IntoView {
    let css = style! {
        
    };

    view! { class=css,
        <div class="wrapper">
            <div class="titleWrapper"></div>
            <div class="contentWrapper"></div>
        </div>
    }
}

pub type EventResult<T> = Result<T, EventError>;

#[derive(Debug)]
pub enum EventError {
    FaultyInitData(serde_json::Error),
}

impl std::error::Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FaultyInitData(v) => {
                write!(
                    f,
                    "Unable to parse initial data to generate Component: {}",
                    v
                )
            }
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(value: serde_json::Error) -> Self {
        Self::FaultyInitData(value)
    }
}
