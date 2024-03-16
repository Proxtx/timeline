use chrono::DateTime;
use chrono::Utc;
use leptos::*;
use stylers::style;
use types::Marker;
use types::TimeRange;

#[component]
fn Timeline(range: Signal<TimeRange>, markers: MaybeSignal<Vec<Marker>>) -> impl IntoView {
    let style = style! {
        .timeline {
            background-color: var(--lightAccentColor1);
            width: 100%;
            overflow: hidden;
            position: relative;
            height: 102px;
        }
    };

    view! { class=style, <div></div> }
}
