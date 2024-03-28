pub struct Plugin {

}

impl crate::Plugin for Plugin {
    async fn new(data: crate::PluginData) -> Self
        where
            Self: Sized {
        Plugin {}
    }

    fn get_compressed_events(
            &self,
            query_range: &types::timing::TimeRange,
        ) -> std::pin::Pin<Box<dyn futures::Future<Output = types::api::APIResult<Vec<types::api::CompressedEvent>>> + Send>> {
        
    }

    fn get_type() -> types::api::AvailablePlugins
        where
            Self: Sized {
        
    }
}