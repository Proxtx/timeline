//! The top "timeline bar" with activity circles and a draggable pointer.

use chrono::{DateTime, SubsecRound, TimeDelta, Timelike, Utc};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use types::timing::{Marker, TimeRange};

use crate::api::api_request;

#[component]
pub fn TimelineBar(
    #[prop(into)] range: Signal<TimeRange>,
    #[prop(into)] on_range_pick: Callback<TimeRange>,
) -> impl IntoView {
    let markers = LocalResource::new(move || {
        let range = range.get();
        async move { api_request::<Vec<Marker>, _>("/markers", &range).await }
    });

    let dragging = RwSignal::new(false);
    let pointer_ref: NodeRef<leptos::html::Img> = NodeRef::new();

    let emit = move |page_x: i32| {
        let Some(win) = web_sys::window() else { return };
        let width = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
        let pct = (page_x as f64 / width) * 100.0;

        if let Some(img) = pointer_ref.get() {
            let el: web_sys::HtmlElement = img.unchecked_into();
            let _ = el.style().set_property("left", &format!("{}%", pct));
        }

        let r = range.get();
        let start_ms = map_range(
            (0.0, 100.0),
            (
                r.start.timestamp_millis() as f64,
                r.end.timestamp_millis() as f64,
            ),
            pct,
        );
        let Some(mut start) = DateTime::<Utc>::from_timestamp_millis(start_ms as i64) else {
            return;
        };
        start = start.round_subsecs(0).with_second(0).unwrap_or(start);
        start = start.with_minute(0).unwrap_or(start);
        let Some(end) = start.checked_add_signed(TimeDelta::try_hours(1).unwrap_or_default()) else {
            return;
        };
        on_range_pick.run(TimeRange { start, end });
    };

    view! {
        <div
            class="timelineBar"
            class:loading=move || markers.get().is_none()
            on:mousedown=move |e| { dragging.set(true); emit(e.page_x()); }
            on:mousemove=move |e| { if dragging.get() { emit(e.page_x()); } }
            on:mouseup=move |_| { dragging.set(false); }
            on:touchstart=move |e| {
                dragging.set(true);
                if let Some(t) = e.touches().item(0) {
                    emit(t.page_x());
                }
            }
            on:touchmove=move |e| {
                if dragging.get() {
                    if let Some(t) = e.touches().item(0) {
                        emit(t.page_x());
                    }
                }
            }
            on:touchend=move |_| { dragging.set(false); }
            on:touchcancel=move |_| { dragging.set(false); }
        >
            {move || markers.get().map(|res| match res {
                Ok(markers) => circles_view(range.get_untracked(), markers),
                Err(e) => view! {
                    <div class="errorWrapper">{format!("Error: {}", e)}</div>
                }.into_any(),
            })}
            <img src="/icons/pointer.svg" class="pointer" node_ref=pointer_ref />
        </div>
    }
}

fn circles_view(range: TimeRange, markers: Vec<Marker>) -> AnyView {
    let max = markers.iter().map(|m| m.amount).max().unwrap_or(1);

    let views: Vec<_> = markers
        .iter()
        .map(|m| {
            let width = format!("{}px", (m.amount as f64 / max as f64) * 100.0);
            let left = format!(
                "{}%",
                map_range(
                    (
                        range.start.timestamp_millis() as f64,
                        range.end.timestamp_millis() as f64,
                    ),
                    (0.0, 100.0),
                    m.time.timestamp_millis() as f64,
                )
            );
            let top = format!("{}%", js_sys::Math::random() * 100.0);
            view! { <div class="circle" style:width=width style:left=left style:top=top /> }
        })
        .collect();

    views.into_any()
}

fn map_range(from: (f64, f64), to: (f64, f64), v: f64) -> f64 {
    to.0 + (v - from.0) * (to.1 - to.0) / (from.1 - from.0)
}
