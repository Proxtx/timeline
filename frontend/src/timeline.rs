use {
    crate::api::api_request,
    chrono::{DateTime, TimeDelta, Utc},
    leptos::{ev::TouchEvent, *},
    rand::Rng,
    stylers::style,
    types::timing::{Marker, TimeRange},
    web_sys::{wasm_bindgen::JsCast, HtmlElement},
};

#[component]
pub fn Timeline(
    #[prop(into)] range: MaybeSignal<TimeRange>,
    #[prop(into)] callback: Callback<TimeRange>,
) -> impl IntoView {
    let resource = create_resource(range.clone(), |range| async move {
        api_request::<Vec<Marker>, _>("/markers", &range).await
    });
    let style = style! {
        @keyframes loading {
            0% {
                opacity: 80%;
            }
            50% {
                opacity: 100%;
            }
            100% {
                opacity: 80%;
            }
        }

        .timeline {
            background-color: var(--accentColor1Light);
            width: 100%;
            overflow: hidden;
            position: relative;
            height: 102px;
        }

        .loading {
            animation: loading 2s;
            animation-iteration-count: infinite;
        }

        .pointer {
            position: absolute;
            transform: translateX(-50%);
            z-index: 1;
            pointer-events: none;
        }
    };
    let pointer_ref: NodeRef<html::Img> = create_node_ref();

    let handle_pointer_event = move |e: TouchEvent, range: &TimeRange| {
        e.prevent_default();
        let pos_percent = e.touches().item(0).unwrap().page_x() as f64
            / leptos::window().inner_width().unwrap().as_f64().unwrap()
            * 100.;
        let _ = pointer_ref().unwrap().style("left", &format!("{}%", pos_percent));

        let start_time_millis = map_range(
            (0., 100.),
            (
                range.start.timestamp_millis() as f64,
                range.end.timestamp_millis() as f64,
            ),
            pos_percent,
        );

        let start_time: DateTime<Utc> =
            DateTime::from_timestamp_millis(start_time_millis as i64).unwrap();
        let end_time = start_time
            .checked_add_signed(TimeDelta::try_hours(1).unwrap())
            .unwrap();
        callback(TimeRange {
            start: start_time,
            end: end_time,
        })
    };

    let range_moved = range.clone();
    let (indicator_is_dragged, set_indicator_is_dragged) = create_signal(false);

    let handle_pointer_event_move = move |e: TouchEvent| {
        if indicator_is_dragged() {
            handle_pointer_event(e, &range_moved.get())
        }
    };

    let range_2 = range.clone();


    view! { class=style,
        <div class="timeline" class:loading=move || resource().is_none() 
            on:touchstart=move |e| {
                set_indicator_is_dragged(true);
                handle_pointer_event(e, &range_2());
            }
            on:touchend=move |_e| { set_indicator_is_dragged.set(false) }
            on:touchcancel=move |_e| { set_indicator_is_dragged.set(false) }
            on:touchmove=handle_pointer_event_move>
            {move || match resource() {
                Some(data) => {
                    match data {
                        Ok(data) => {
                            let range_3 = range.clone();
                                (move || get_circles(&range_3(), &data)).into_view()
                        }
                        Err(e) => {
                            view! {
                                <div class="errorWrapper">
                                    {move || { format!("Error loading timeline-bubbles: {}", e) }}
                                </div>
                            }
                                .into_view()
                        }
                    }
                }
                None => ().into_view(),
            }}

            <img
                src="/icons/pointer.svg"
                class="pointer"
                node_ref = pointer_ref
            />
        </div>
    }
}

pub fn get_circles(range: &TimeRange, markers: &[Marker]) -> impl IntoView {
    let mut max = u32::MIN;
    let mut min = u32::MAX;

    for marker in markers.iter() {
        if marker.amount > max {
            max = marker.amount
        }
        if marker.amount < min {
            min = marker.amount
        }
    }

    let style = style! {
        @keyframes popIn {
            0% {
                transform: translate(-50%, -50%) scale(0);
            }
            100% {
                transform: translate(-50%, -50%) scale(1);
            }
        }

        .circle {
            position: absolute;
            background-color: #006ba39c;
            aspect-ratio: 1;
            border-radius: 50%;
            transform: translate(-50%, -50%);
            animation: popIn 1s;
        }
    };

    view! {
        {markers
            .iter()
            .map(|m| {
                let width = format!("{}px", m.amount as f64 / max as f64 * 100_f64);
                let left = format!(
                    "{}%",
                    map_range(
                        (
                            range.start.timestamp_millis() as f64,
                            range.end.timestamp_millis() as f64,
                        ),
                        (0., 100.),
                        m.time.timestamp_millis() as f64,
                    ),
                );
                let mut rand = rand::thread_rng();
                let top = format!("{}%", rand.gen_range(0.0..100.0));
                view! { class=style,
                    <div class="circle" style:width=width style:left=left style:top=top></div>
                }
            })
            .collect::<Vec<_>>()}
    }
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
