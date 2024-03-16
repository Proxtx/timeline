use chrono::DateTime;
use chrono::Utc;
use leptos::*;
use stylers::style;
use types::timing::Marker;
use types::timing::TimeRange;
use crate::api::api_request;

#[component]
fn Timeline(range: MaybeSignal<TimeRange>, callback: Callback<TimeRange>) -> impl IntoView {
    let resource = create_resource(range, |range| async move {
        api_request::<Vec<Marker>, _>("/marker", &range).await
    });
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
