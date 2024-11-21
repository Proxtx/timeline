pub trait Plugin: Send + Sync {
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