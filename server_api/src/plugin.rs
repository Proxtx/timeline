use std::{pin::Pin, sync::Arc};

use {
    rocket::{Build, Rocket, Route},
    types::{
        api::CompressedEvent, available_plugins::AvailablePlugins, external::chrono::Duration,
        timing::TimeRange,
    },
    url::Url,
};

use crate::db::Database;

pub trait PluginTrait: Send + Sync {
    fn new(data: PluginData) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_type() -> AvailablePlugins
    where
        Self: Sized;

    fn request_loop_mut<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn futures::Future<Output = Option<Duration>> + Send + 'a>> {
        Box::pin(async move { None })
    }

    fn request_loop<'a>(
        &'a self,
    ) -> Pin<Box<dyn futures::Future<Output = Option<Duration>> + Send + 'a>> {
        Box::pin(async move { None })
    }

    fn get_compressed_events(
        &self,
        query_range: &TimeRange,
    ) -> Pin<Box<dyn futures::Future<Output = types::api::APIResult<Vec<CompressedEvent>>> + Send>>;

    fn get_routes() -> Vec<Route>
    where
        Self: Sized,
    {
        Vec::new()
    }

    fn rocket_build_access(&self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket
    }
}

pub struct PluginData {
    pub database: Arc<Database>,
    pub config: Option<toml::Value>,
    pub plugin: AvailablePlugins,
    pub error_url: Option<Url>,
}

impl PluginData {
    pub fn report_error(&self, error: &impl std::error::Error) {
        crate::error::error(
            self.database.clone(),
            error,
            Some(self.plugin.clone()),
            &self.error_url,
        )
    }

    pub fn report_error_string(&self, string: String) {
        crate::error::error_string(
            self.database.clone(),
            string,
            Some(self.plugin.clone()),
            &self.error_url,
        )
    }
}
