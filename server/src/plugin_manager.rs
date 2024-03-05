use std::collections::HashMap;
use std::sync::Arc;
use tokio::pin;
use tokio::sync::RwLock;

use crate::Plugin;

type ThreadedPlugin<'a> = Arc<RwLock<Box<dyn Plugin<'a>>>>;
type PluginsMap<'a> = HashMap<String, ThreadedPlugin<'a>>;
pub struct PluginManager<'a> {
    plugins: PluginsMap<'a>,
}

impl<'a> PluginManager<'a> {
    pub fn new(plugins: HashMap<String, Box<dyn Plugin<'a>>>) -> Self {
        let plugins: PluginsMap = plugins
            .into_iter()
            .map(|(key, value)| (key, Arc::new(RwLock::new(value))))
            .collect();
        PluginManager { plugins }
    }

    pub async fn update_loop<'b>(plugin: ThreadedPlugin<'b>) -> Option<chrono::Duration> {
        /*let mut mut_plg = plugin.write().await;
        let fut = mut_plg.request_loop();
        pin!(fut);
        return fut.await;*/
        return None;
    }
}
