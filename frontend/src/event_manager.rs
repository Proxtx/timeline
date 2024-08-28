use {
    crate::{
        api::api_request, events_display::{DefaultEventsViewerType, EventsViewer}, plugin_manager::PluginManager, APIResult, StyledView
    }, experiences_navigator_lib::experiences_types::types::ExperiencesHostname, leptos::*, std::{collections::HashMap, sync::Arc}, timeline_frontend_lib::{events_display::DefaultWithAvailablePluginsEventsViewerType, plugin_manager}, types::{
        api::{AvailablePlugins, CompressedEvent},
        timing::TimeRange,
    }
};

#[cfg(feature="experiences")]
use {
    experiences_navigator_lib::{navigator::StandaloneNavigator, wrappers::Band},
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
                                    plugin.clone(),
                                    events
                                        .into_iter()
                                        .filter(|current_event| {
                                            current_range().overlap_timing(&current_event.time)
                                        }).map(|v| (plugin.clone(), v))
                                        .collect::<Vec<(AvailablePlugins, CompressedEvent)>>(),
                                )
                            })
                            .filter(|(_plugin, data)| !data.is_empty())
                            .collect::<HashMap<AvailablePlugins, Vec<(AvailablePlugins, CompressedEvent)>>>()
        );

        let mut slide_over: Option<DefaultWithAvailablePluginsEventsViewerType> = None;

        #[cfg(feature="experiences")]
        {
            slide_over  = Some(|event, close_callback| {
                let selected_experience = create_rw_signal(None);
                let close_callback = Arc::new(close_callback);
                let close_callback_2 = close_callback.clone();

                view! {
                    <StyledView>
                        <Band click=Callback::new(move |_| {
                            close_callback_2();
                        })>
                            <b>Close</b>
                        </Band>
                        <StandaloneNavigator selected_experience=selected_experience/>
                        <Band click=Callback::new(move |_| {
                            spawn_local({
                                let close_callback = close_callback.clone();
                                let selected_experience = selected_experience();
                                let event = event.clone();
                                async move {
                                    close_callback();
                                    if let Err(e) = experiences_navigator_lib::api::api_request::<
                                        String,
                                        _,
                                    >(
                                            &format!(
                                                "/experience/{}/append_event",
                                                selected_experience.unwrap(),
                                            ),
                                            &event,
                                        )
                                        .await
                                    {
                                        window()
                                            .alert_with_message(
                                                &format!("Unable to append event to experience: {}", e),
                                            )
                                            .unwrap();
                                    }
                                }
                            })
                        })>
                            <b>Insert</b>
                        </Band>
                    </StyledView>
                }.into_view()
            }); 
        }

        type AVCTuple = (AvailablePlugins, CompressedEvent);

        view! {
            <EventsViewer<AVCTuple, DefaultWithAvailablePluginsEventsViewerType>
                events=current_events
                plugin_manager=plugin_manager.clone()
                slide_over=slide_over
            />
        }
    }