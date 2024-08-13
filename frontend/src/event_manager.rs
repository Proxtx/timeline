use {
    crate::{
        api::api_request,
        plugin_manager::PluginManager,
        events_display::EventViewer
    },
    std::collections::HashMap,
    leptos::*,
    types::{
        api::{APIResult, AvailablePlugins, CompressedEvent},
        timing::TimeRange,
    },
};

#[component]
pub fn EventManager(
    #[prop(into)] available_range: MaybeSignal<TimeRange>,
    #[prop(into)] current_range: MaybeSignal<TimeRange>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
) -> impl IntoView {
let available_events = create_resource(available_range, |range| async move {
        logging::log!("reloading all events");
        api_request::<HashMap<AvailablePlugins, Vec<CompressedEvent>>, _>("/events", &range).await
    });

    let current_events =
        create_memo(
            move |_: Option<&APIResult<_>>| match available_events.get() {
                Some(available_events) => {
                    let available_events = available_events?;
                    Ok(Some(
                        available_events
                            .into_iter()
                            .map(|(plugin, events)| {
                                (
                                    plugin,
                                    events
                                        .into_iter()
                                        .filter(|current_event| {
                                            current_range().overlap_timing(&current_event.time)
                                        })
                                        .collect::<Vec<CompressedEvent>>(),
                                )
                            })
                            .filter(|(_plugin, data)| !data.is_empty())
                            .collect::<HashMap<AvailablePlugins, Vec<CompressedEvent>>>(),
                    ))
                }
                None => Ok(None),
            },
        );

        view! {
            {move || match current_events() {
                Ok(v) => {
                    match v {
                        Some(v) => {
                            view! { <EventViewer events=v plugin_manager=plugin_manager.clone() /> }
                                .into_view()
                        }
                        None => view! { <div class="infoWrapper">Loading</div> }.into_view(),
                    }
                }
                Err(e) => {
                    view! {
                        <div class="errorWrapper">
                            {move || format!("Error loading app selector: {}", e)}
                        </div>
                    }
                        .into_view()
                }
            }}
        }
    }