use {
    crate::{
        api::api_request,
        plugin_manager::PluginManager,
        events_display::EventsViewer
    },
    std::collections::HashMap,
    leptos::*,
    types::{
        api::{AvailablePlugins, CompressedEvent},
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

        view! {
            {move || match available_events() {
                Some(v) => {
                    match v {
                        Ok(v) => {
                            view! {
                                <DisplayCurrentEvents
                                    available_events=v
                                    current_range=current_range.clone()
                                    plugin_manager=plugin_manager.clone()
                                />
                            }
                                .into_view()
                        }
                        Err(e) => {
                            view! {
                                <div class="errorWrapper">
                                    {move || {
                                        format!("Error loading events display selector: {}", e)
                                    }}

                                </div>
                            }
                                .into_view()
                        }
                    }
                }
                None => view! { <div class="infoWrapper">Loading</div> }.into_view(),
            }}
        }
    }

    #[component]
    fn DisplayCurrentEvents(
        #[prop(into)] available_events: MaybeSignal<HashMap<AvailablePlugins, Vec<CompressedEvent>>>,
        #[prop(into)] current_range: MaybeSignal<TimeRange>,
        #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
    ) -> impl IntoView{
        let current_events =
        create_memo(
            move |_| available_events()
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
                            .collect::<HashMap<AvailablePlugins, Vec<CompressedEvent>>>()
        );

        view! { <EventsViewer events=current_events plugin_manager=plugin_manager.clone()/> }
    }