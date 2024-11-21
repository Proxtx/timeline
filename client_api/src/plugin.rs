pub trait Plugin {
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
