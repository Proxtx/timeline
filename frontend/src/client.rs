pub struct Plugin {}

impl crate::Plugin for Plugin {
    async fn new(
        _data: crate::plugin_manager::PluginData,
    ) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }
}
