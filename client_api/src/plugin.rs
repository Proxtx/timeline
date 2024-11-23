use crate::result::EventResult;
use crate::style::Style;
use leptos::View;
use std::fmt;
use types::{
    api::CompressedEvent,
    external::{chrono::naive::serde, serde::de::DeserializeOwned, serde_json},
};
use url::Url;

pub struct PluginData {}

pub trait PluginTrait {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_component(&self, data: PluginEventData) -> EventResult<Box<dyn FnOnce() -> View>>;

    fn get_style(&self) -> Style;

    fn get_icon(&self) -> IconLocation {
        IconLocation::Default
    }

    fn get_events_overview(&self, events: &Vec<CompressedEvent>) -> Option<View> {
        None
    }
}

pub enum IconLocation {
    Default,
    Custom(Url),
}

pub struct PluginEventData<'a> {
    pub data: &'a serde_json::Value,
}

impl<'a> PluginEventData<'a> {
    pub fn get_data<T>(&self) -> EventResult<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_value(self.data.clone())?)
    }

    pub fn get_raw(&self) -> &serde_json::Value {
        self.data
    }
}
