use std::{collections::HashMap, fmt};

use leptos::*;
use types::{
    api::{AvailablePlugins, CompressedEvent},
    timing::TimeRange,
};

use crate::api::api_request;

#[component]
fn event_manager(
    available_range: MaybeSignal<TimeRange>,
    current_range: MaybeSignal<TimeRange>,
) -> impl IntoView {
    let available_events = create_resource(available_range, |range| async move {
        logging::log!("reloading all events");
        api_request::<HashMap<AvailablePlugins, CompressedEvent>, _>("/get_events", &range).await
    });

    



    view! {

    }
}

pub type EventResult<T> = Result<T, EventError>;

#[derive(Debug)]
pub enum EventError {
    FaultyInitData(serde_json::Error),
}

impl std::error::Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FaultyInitData(v) => {
                write!(
                    f,
                    "Unable to parse initial data to generate Component: {}",
                    v
                )
            }
        }
    }
}

impl From<serde_json::Error> for EventError {
    fn from(value: serde_json::Error) -> Self {
        Self::FaultyInitData(value)
    }
}
