use std::ops::Deref;

use chrono::DateTime;
use chrono::Utc;
use leptos::*;
use stylers::style;
use types::timing::Marker;
use types::timing::TimeRange;
use web_sys::js_sys::Float32Array;
use web_sys::wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use crate::api::api_request;
use rand::Rng;
use leptos::ev::TouchEvent;

#[component]
pub fn Timeline(#[prop(into)] range: MaybeSignal<TimeRange>, #[prop(into)] callback: Callback<TimeRange>) -> impl IntoView {
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
        }
    };

    let handle_pointer_event = |e: TouchEvent| {
        let pos_percent = e.touches().item(0).unwrap().page_x() as f64 / leptos::window().inner_width().unwrap().as_f64().unwrap() * 100.;
        e.target().unwrap().dyn_into::<HtmlElement>().unwrap().style().set_property("left", &format!("{}%", pos_percent)).unwrap();
        //let range = map_range(from_range, to_range, s)
        //callback
    };

    let (indicator_is_dragged, set_indicator_is_dragged) = create_signal(false);

    view! { class=style,
        <div class="timeline" class:loading=move || resource().is_none()>

            {move || match resource.get() {
                None => view! { <a>"Loading"</a> }.into_view(),
                Some(data) => view! { {get_circles(&range(), &data.unwrap())} }.into_view(),
            }}

            <img
                src="/icons/pointer.svg"
                class="pointer"
                on:touchstart=move |e| {
                    handle_pointer_event(e);
                    set_indicator_is_dragged.set(true)
                }

                on:touchend=move |e| { set_indicator_is_dragged.set(false) }

                on:touchcancel=move |e| { set_indicator_is_dragged.set(false) }

                on:touchmove=move |e| {
                    if indicator_is_dragged() {
                        handle_pointer_event(e);
                    }
                }

                on:mousemove=move |e| { logging::log!("{}", e.page_x()) }
            />

        </div>
    }
}

pub fn get_circles(
    range: &TimeRange,
    markers: &[Marker]
) -> impl IntoView {
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
        .circle {
            position: absolute;
            background-color: #006ba39c;
            aspect-ratio: 1;
            border-radius: 50%;
            transform: translate(-50%, -50%);
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