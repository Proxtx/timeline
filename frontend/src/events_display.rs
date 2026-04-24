//! Event list UI. For each event we mount the plugin-supplied shadow-DOM
//! widget into a host element once expanded.

use std::collections::HashMap;

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use types::api::CompressedEvent;

use crate::plugin_manager::PluginManager;

#[derive(Clone, Copy)]
pub struct DisplayWithDay(pub bool);

type EventMap = HashMap<String, Vec<CompressedEvent>>;

#[component]
pub fn EventsViewer(
    #[prop(into)] events: Signal<EventMap>,
    #[prop(into)] plugin_manager: Signal<PluginManager>,
) -> impl IntoView {
    let available_plugins = Memo::new(move |_| {
        let mut plugins: Vec<String> = events.with(|e| e.keys().cloned().collect());
        plugins.sort();
        plugins
    });

    let current_app: RwSignal<Option<String>> = RwSignal::new(None);

    // Auto-select first app if none chosen.
    Effect::new(move |_| {
        if current_app.with(Option::is_none) {
            let first = available_plugins.with(|v| v.first().cloned());
            if let Some(name) = first {
                current_app.set(Some(name));
            }
        }
    });

    let current_events = Memo::new(move |_| {
        let app = current_app.get();
        app.and_then(|name| events.with(|e| e.get(&name).cloned()))
            .unwrap_or_default()
    });

    let plugins_signal = Signal::derive(move || available_plugins.get());
    let events_signal = Signal::derive(move || current_events.get());
    view! {
        <AppSelect plugins=plugins_signal current_app=current_app manager=plugin_manager />
        {move || match current_app.get() {
            Some(name) => view! {
                <EventsDisplay plugin_name=name events=events_signal manager=plugin_manager />
            }.into_any(),
            None => view! { <div class="infoWrapper">No App Selected</div> }.into_any(),
        }}
    }
}

#[component]
fn AppSelect(
    #[prop(into)] plugins: Signal<Vec<String>>,
    current_app: RwSignal<Option<String>>,
    #[prop(into)] manager: Signal<PluginManager>,
) -> impl IntoView {
    view! {
        <div class="appSelector">
            <For
                each={move || plugins.get()}
                key={|name: &String| name.clone()}
                children={move |name: String| {
                    let icon_url = manager.with(|m| m.icon_url(&name));
                    let name_for_click = name.clone();
                    let name_for_indicator = name.clone();
                    let name_for_color = name.clone();
                    view! {
                        <div class="iconWrap">
                            <img
                                src=icon_url
                                class="appIcon"
                                on:click=move |_| { current_app.set(Some(name_for_click.clone())); }
                            />
                            <div
                                class="indicator"
                                style:height=move || {
                                    if current_app.get().as_deref() == Some(name_for_indicator.as_str()) {
                                        "100%"
                                    } else {
                                        "0%"
                                    }
                                }
                                style:background-color=move || manager.with(|m| m.style(&name_for_color).bg())
                            />
                        </div>
                    }
                }}
            />
        </div>
    }
}

#[component]
fn EventsDisplay(
    plugin_name: String,
    #[prop(into)] events: Signal<Vec<CompressedEvent>>,
    #[prop(into)] manager: Signal<PluginManager>,
) -> impl IntoView {
    let name_for_bg = plugin_name.clone();
    let bg = move || manager.with(|m| m.style(&name_for_bg).bg());
    let name_for_children = plugin_name.clone();
    view! {
        <div class="eventsList" style:background-color=bg>
            <For
                each={move || events.get().into_iter().enumerate().collect::<Vec<_>>()}
                key={|(idx, ev): &(usize, CompressedEvent)| format!("{}-{}", idx, ev.title)}
                children={move |(_idx, event): (usize, CompressedEvent)| {
                    view! {
                        <EventDisplay
                            plugin_name=name_for_children.clone()
                            event=event
                            manager=manager
                        />
                    }
                }}
            />
        </div>
    }
}

#[component]
fn EventDisplay(
    plugin_name: String,
    event: CompressedEvent,
    #[prop(into)] manager: Signal<PluginManager>,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let display_with_day = use_context::<DisplayWithDay>().map(|d| d.0).unwrap_or(false);

    let time_label = if display_with_day {
        event.time.display_with_day()
    } else {
        format!("{}", event.time)
    };

    let name_for_header = plugin_name.clone();
    let header_color = move || manager.with(|m| m.style(&name_for_header).text().to_string());
    let name_for_border = plugin_name.clone();
    let border_color = move || manager.with(|m| m.style(&name_for_border).light());
    let name_for_body = plugin_name.clone();
    let body_color = move || manager.with(|m| m.style(&name_for_body).light());

    let host_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let mounted = RwSignal::new(false);

    let event_for_effect = event.clone();
    let plugin_for_effect = plugin_name.clone();
    Effect::new(move |_| {
        if !expanded.get() || mounted.get_untracked() {
            return;
        }
        let Some(host) = host_ref.get() else { return };
        let host_el: web_sys::HtmlElement = host.unchecked_into();
        let manager = manager.get_untracked();
        let name = plugin_for_effect.clone();
        let ev = event_for_effect.clone();
        mounted.set(true);
        spawn_local(async move {
            manager.mount(&name, host_el, ev).await;
        });
    });

    let event_title = event.title.clone();

    view! {
        <div class="eventRow" style:border-top=move || format!("1px solid {}", border_color())>
            <button
                class="eventHeader"
                style:color=header_color
                on:click=move |_| expanded.update(|v| *v = !*v)
            >
                <h3>{event_title}</h3>
                <a>{time_label}</a>
            </button>
            <div
                class="eventBody"
                style:background-color=body_color
                style:display=move || if expanded.get() { "block" } else { "none" }
            >
                <div class="pluginHost" node_ref=host_ref />
            </div>
        </div>
    }
}
