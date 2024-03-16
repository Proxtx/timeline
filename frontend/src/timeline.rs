use chrono::DateTime;
use chrono::Utc;
use leptos::*;
use stylers::style;
use types::timing::Marker;
use types::timing::TimeRange;
use crate::api::api_request;

#[component]
pub fn Timeline(#[prop(into)] range: MaybeSignal<TimeRange>, #[prop(into)] callback: Callback<TimeRange>) -> impl IntoView {
    let resource = create_resource(range, |range| async move {
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
    };

    view! { class=style,
        <div class="timeline" class:loading=move || resource().is_none()>

            {move || match resource.get() {
                None => view! { <a>"Loading"</a> }.into_view(),
                Some(data) => view! { {format!("{:?}", data)} }.into_view(),
            }}

        </div>
    }
}

/*pub fn get_circles(
    range: &TimeRange,
    markers: &[Marker]
) -> impl IntoView {
    let max = u64::MIN;
    let min = u64::MAX;


    style! {
        .circle {
            position: absolute;
            background-color: 
        }
    }
    view! { {markers.iter().map(|m| view! { <div></div> })} }
}
*/