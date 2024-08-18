use {
    crate::{
        api::relative_url,
        plugin_manager::{EventResult, IconLocation, PluginManager, Style},
    },
    leptos::*,
    std::{
        collections::HashMap,
        hash::{DefaultHasher, Hash, Hasher},
    },
    stylers::style,
    types::api::{APIResult, AvailablePlugins, CompressedEvent, EventWrapper},
};

pub type DefaultEventsViewerType = fn(CompressedEvent) -> View;

#[component]
pub fn EventsViewer<T: EventWrapper>(
    #[prop(into)] events: MaybeSignal<HashMap<AvailablePlugins, Vec<T>>>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
    #[prop(into, default=None.into())] slide_over: Option<impl Fn(T) -> View + Clone + 'static>,
) -> impl IntoView {
    let plugin_manager_e = plugin_manager.clone();

    let events_sel = events.clone();

    let currently_available_plugins = move || {
        let mut plugins = events().keys().cloned().collect::<Vec<AvailablePlugins>>();
        plugins.sort_by(|s, o| format!("{}", s).cmp(&format!("{}", o)));
        APIResult::Ok(Some(plugins))
    };

    let events = events_sel;

    let current_app: RwSignal<Option<AvailablePlugins>> = create_rw_signal(None);

    let selected_events = create_memo(move |_| match current_app() {
        Some(app) => match events().get(&app) {
            Some(v) => v.clone(),
            None => Vec::new(),
        },
        None => Vec::new(),
    });

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
            }
        }}

        {move || {
            match current_app() {
                Some(plugin) => {
                    view! {
                        <EventsDisplay
                            plugin=plugin
                            selected_events=selected_events
                            plugin_manager=plugin_manager_e.clone()
                            slide_over=slide_over.clone()
                        />
                    }
                }
                None => view! { <div class="infoWrapper">No App Selected</div> }.into_view(),
            }
        }}
    }
}

#[component]
fn AppSelect(
    #[prop(into)] selectable_apps: MaybeSignal<Vec<AvailablePlugins>>,
    #[prop(into)] current_app: RwSignal<Option<AvailablePlugins>>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
) -> impl IntoView {
    let style = style! {
        .selector {
            --padding: calc(var(--contentSpacing) * 1.5);
            height: calc(50px + 2 * var(--padding));
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: safe center;
            padding: var(--padding);
            background-color: var(--darkColor);
            box-sizing: border-box;
            overflow: hidden;
            overflow-x: auto;
            gap: var(--contentSpacing);
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
                    let url = match plugin_manager().get_icon(&t) {
                        IconLocation::Default => {
                            relative_url("/api/icon/").unwrap().join(&format!("{}", t)).unwrap()
                        }
                        IconLocation::Custom(v) => v,
                    };
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
fn EventsDisplay<T: EventWrapper>(
    #[prop(into)] plugin: MaybeSignal<AvailablePlugins>,
    #[prop(into)] selected_events: MaybeSignal<Vec<T>>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
    #[prop(into)] slide_over: MaybeSignal<Option<impl Fn(T) -> View + Clone + 'static>>,
) -> impl IntoView {
    let css = style! {
        .wrapper {
            flex: 1 0;
            transition: 0.2s;
            overflow: auto;
        }
    };

    let plugin_manager_d = plugin_manager.clone();
    let plugin_d = plugin.clone();

    view! { class=css,
        <div
            class="wrapper"
            style:background-color=move || { format!("{}", plugin_manager().get_style(&plugin())) }
        >

            <For
                each=selected_events
                key=|e| {
                    let mut hasher = DefaultHasher::new();
                    e.hash(&mut hasher);
                    hasher.finish()
                }

                children=move |e| {
                    view! {
                        <EventDisplay
                            event=e
                            plugin_manager=plugin_manager_d.clone()
                            slide_over=slide_over.clone()
                            plugin=plugin_d.clone()
                        />
                    }
                }
            />

        </div>
    }
}

#[component]
pub fn EventDisplay<T: EventWrapper>(
    #[prop(into)] event: MaybeSignal<T>,
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
    #[prop(into)] plugin: MaybeSignal<AvailablePlugins>,
    #[prop(default=create_rw_signal(false))] expanded: RwSignal<bool>,
    #[prop(into)] slide_over: MaybeSignal<Option<impl Fn(T) -> View + Clone + 'static>>,
) -> impl IntoView {
    let css = style! {
        .wrapper:first-child {
            border-top: none !important;
        }
        .titleWrapper {
            color: var(--lightColor);
            padding: calc(var(--contentSpacing) * 0.7);
            display: flex;
            flex-direction: column;
            background: none;
            border: none;
            font-family: Rubik;
            font-size: unset;
            width: 100%;
        }
    };

    let event_unwrapped = move || event.with(|v| v.get_compressed_event());
    let event_unwrapped_2 = event_unwrapped.clone();
    let event_unwrapped_3 = event_unwrapped.clone();

    let plugin_manager_2 = plugin_manager.clone();
    let plugin_manager_3 = plugin_manager.clone();

    let plugin_2 = plugin.clone();
    let plugin_3 = plugin.clone();

    view! { class=css,
        <div
            class="wrapper"
            style:border-top=move || {
                format!("1px solid {}", plugin_manager_2().get_style(&plugin_2()).light())
            }
        >

            <button
                class="titleWrapper"
                on:click=move |_| expanded.set(!expanded.get())
                style:color=move || { plugin_manager_3().get_style(&plugin_3()).text().to_string() }
            >
                <h3>{move || event_unwrapped_2().title}</h3>
                <a>{move || format!("{}", event_unwrapped_3().time)}</a>
            </button>
            <EventContent
                plugin_manager=plugin_manager
                data=Signal::derive(move || { event_unwrapped().data })
                plugin=plugin
                expanded=expanded
            />
        </div>
    }
}

#[component]
fn EventContent(
    #[prop(into)] plugin_manager: MaybeSignal<PluginManager>,
    #[prop(into)] plugin: MaybeSignal<AvailablePlugins>,
    #[prop(into)] data: MaybeSignal<String>,
    #[prop(into)] expanded: MaybeSignal<bool>,
) -> impl IntoView {
    let plugin_manager_2 = plugin_manager.clone();
    let plugin_2 = plugin.clone();
    let style = move || plugin_manager_2().get_style(&plugin_2());

    let (read_view, write_view) = create_signal(None);
    view! {
        {move || match (expanded(), read_view()) {
            (true, Some(v)) => {
                view! { <ShowResultEventView style=Signal::derive(style.clone()) view=v/> }
                    .into_view()
            }
            (true, None) => {
                data.with(|d| {
                    match plugin_manager().get_component(&plugin(), d) {
                        Ok(v) => {
                            write_view(Some(Ok(v())));
                        }
                        Err(e) => {
                            write_view(Some(Err(e)));
                        }
                    }
                    view! {
                        <ShowResultEventView
                            style=Signal::derive(style.clone())
                            view=read_view().unwrap()
                        />
                    }
                        .into_view()
                })
            }
            (false, _) => ().into_view(),
        }}
    }
}

#[component]
fn ShowResultEventView(
    #[prop(into)] view: MaybeSignal<EventResult<View>>,
    #[prop(into)] style: MaybeSignal<Style>,
) -> impl IntoView {
    let css = style! {
        .wrapper {
            width: 100%;
            position: relative;
            padding: var(--contentSpacing);
            box-sizing: border-box;
        }
    };
    view! { class=css,
        <div class="wrapper" style:background-color=move || { style().light().to_string() }>

            {move || match view() {
                Ok(v) => v,
                Err(e) => format!("{}", e).into_view(),
            }}

        </div>
    }
}
