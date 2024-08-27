use {
    crate::{
        api::api_request, events_display::{DefaultEventsViewerType, EventsViewer}, plugin_manager::PluginManager
    }, experiences_navigator_lib::experiences_types::types::ExperiencesHostname, leptos::*, std::collections::HashMap, timeline_frontend_lib::plugin_manager, types::{
        api::{AvailablePlugins, CompressedEvent},
        timing::TimeRange,
    }
};

#[cfg(feature="experiences")]
use {
    experiences_navigator_lib::navigator::StandaloneNavigator,
};

#[component]
pub fn EventManager(
    #[prop(into)] available_range: MaybeSignal<TimeRange>,
    #[prop(into)] current_range: MaybeSignal<TimeRange>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
) -> impl IntoView {

    let experiences_url_error = create_resource(
        || {},
        |_| async {
            let mut res = None;
            #[cfg(feature="experiences")]
            {
                res = match api_request::<String, _>("/experiences_url", &()).await {
                    Ok(v) => {
                        provide_context(ExperiencesHostname(v));
                        None
                    }
                    Err(e) => Some(e),
                };
            }
            res
        },
    );

    let available_events = create_resource(available_range, |range| async move {
        logging::log!("reloading all events");
        api_request::<HashMap<AvailablePlugins, Vec<CompressedEvent>>, _>("/events", &range).await
    });


        view! {
            {move || {
                let current_range = current_range.clone();
                let plugin_manager = plugin_manager.clone();
                match experiences_url_error() {
                    Some(experiences_url_error) => {
                        match experiences_url_error {
                            None => {
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
                                                    {
                                                        view! {
                                                            <div class="errorWrapper">

                                                                {move || {
                                                                    format!("Error loading events display selector: {}", e)
                                                                }}

                                                            </div>
                                                        }
                                                    }
                                                        .into_view()
                                                }
                                            }
                                        }
                                        None => {
                                            view! { <div class="infoWrapper">Loading</div> }.into_view()
                                        }
                                    }}
                                }
                                    .into_view()
                            }
                            Some(e) => {
                                view! {
                                    <div class="errorWrapper">
                                        Error loading Experiences Url: {e.to_string()}
                                    </div>
                                }
                                    .into_view()
                            }
                        }
                    }
                    None => view! { <div class="infoWrapper">Loading</div> }.into_view(),
                }
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

        let mut slide_over: Option<DefaultEventsViewerType> = None;

        #[cfg(feature="experiences")]
        {
            slide_over  = Some(|event, close_callback| {
                view! { <StandaloneNavigator/> }.into_view()
            }); 
        }

        view! {
            <EventsViewer<CompressedEvent, DefaultEventsViewerType>
                events=current_events
                plugin_manager=plugin_manager.clone()
                slide_over=slide_over
            />
        }
    }