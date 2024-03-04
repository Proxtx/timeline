use std::collections::HashMap;
use std::sync::Arc;
use tokio::pin;
use tokio::sync::RwLock;

use crate::Plugin;

type ThreadedPlugin<'a> = Arc<RwLock<Box<dyn Plugin<'a> + Send + Sync>>>;
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

    pub async fn update_loop<'b>(plugin: ThreadedPlugin<'b>)
    where
        Self: Send,
    {
        let rqwlp;
        {
            let mut mut_plg = plugin.write().await;
            let fut = mut_plg.request_loop();
            pin!(fut);
            rqwlp = fut.await;
        }
        match rqwlp {
            Some(v) => {
                tokio::time::sleep(
                    v.to_std().unwrap(), /* why should this fail? If it fails is will probably during testing. */
                );
                tokio::spawn(async move { PluginManager::update_loop(plugin).await });
            }
            _ => {}
        }
    }
}
