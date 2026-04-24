//! Event fetching + filtering to a time range, then hand off to events_display.

use std::collections::HashMap;

use leptos::prelude::*;

use types::api::CompressedEvent;
use types::timing::TimeRange;

use crate::api::api_request;
use crate::events_display::EventsViewer;
use crate::plugin_manager::PluginManager;

type EventMap = HashMap<String, Vec<CompressedEvent>>;

#[component]
pub fn EventManager(
    #[prop(into)] available_range: Signal<TimeRange>,
    #[prop(into)] current_range: Signal<TimeRange>,
    #[prop(into)] plugin_manager: Signal<PluginManager>,
) -> impl IntoView {
    let available_events = LocalResource::new(move || {
        let range = available_range.get();
        async move {
            leptos::logging::log!("reloading events");
            api_request::<EventMap, _>("/events", &range).await
        }
    });

    let filtered = Memo::new(move |_| -> Option<Result<EventMap, String>> {
        let inner = available_events.get()?;
        match inner.clone() {
            Ok(all) => {
                let range = current_range.get();
                let out: EventMap = all
                    .into_iter()
                    .map(|(plugin, events)| {
                        let kept: Vec<CompressedEvent> = events
                            .into_iter()
                            .filter(|e| range.overlap_timing(&e.time))
                            .collect();
                        (plugin, kept)
                    })
                    .filter(|(_, e)| !e.is_empty())
                    .collect();
                Some(Ok(out))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    });

    view! {
        {move || match filtered.get() {
            Some(Ok(map)) => view! {
                <EventsViewer
                    events=Signal::derive(move || map.clone())
                    plugin_manager=plugin_manager
                />
            }.into_any(),
            Some(Err(e)) => view! {
                <div class="errorWrapper">{format!("Error loading events: {}", e)}</div>
            }.into_any(),
            None => view! { <div class="infoWrapper">Loading...</div> }.into_any(),
        }}
    }
}
