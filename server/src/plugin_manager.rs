use futures::{FutureExt, StreamExt};
use types::api::APIResult;
use types::timing::TimeRange;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::pin;
use tokio::sync::RwLock;

use crate::{AvailablePlugins, CompressedEvent, Plugin};

type ThreadedPlugin = Arc<RwLock<Box<dyn Plugin>>>;
type PluginsMap = HashMap<AvailablePlugins, ThreadedPlugin>;
pub struct PluginManager {
    plugins: PluginsMap,
}

impl PluginManager {
    pub fn new(plugins: HashMap<AvailablePlugins, Box<dyn Plugin>>) -> Self {
        let plugins: PluginsMap = plugins
            .into_iter()
            .map(|(key, value)| (key, Arc::new(RwLock::new(value))))
            .collect();
        for (_, plg) in plugins.iter() {
            let plg = plg.clone();
            let plg_mut = plg.clone();
            tokio::spawn(async move {
                PluginManager::update_loop_mut(plg_mut).await;
            });
            tokio::spawn(async move {
                PluginManager::update_loop(plg).await;
            });
        }
        PluginManager { plugins }
    }

    pub async fn get_compress_events(&self, time_range: &TimeRange) -> APIResult<HashMap<AvailablePlugins, Vec<crate::CompressedEvent>>> {
        let mut futures = futures::stream::FuturesUnordered::new();
        for (name, plugin) in self.plugins.iter() {
            futures.push(async move{
                (name.clone(), plugin.read().await.get_compressed_events(&time_range).await)
            })
        }

        let mut app_events = HashMap::new();

        while let Some((name, compressed_events)) = futures.next().await {
            let mut evt = compressed_events?;
            evt.sort_by(|s, o| s.time.cmp(&o.time));
            app_events.insert(name, evt);
        }

        Ok(app_events)
    }

    pub fn update_loop_mut(
        plugin: ThreadedPlugin,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let lptm;
            {
                let mut mut_plg = plugin.write().await;
                let fut = mut_plg.request_loop_mut();
                lptm = fut.await;
            }
            if let Some(v) = lptm {
                tokio::time::sleep(v.to_std().unwrap()).await;
                tokio::spawn(async move {
                    PluginManager::update_loop_mut(plugin).await;
                });
            }
        }
        .boxed()
    }

    pub fn update_loop(
        plugin: ThreadedPlugin,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let lptm;
            {
                let mut_plg = plugin.read().await;
                let fut = mut_plg.request_loop();
                lptm = fut.await;
            }
            if let Some(v) = lptm {
                tokio::time::sleep(v.to_std().unwrap()).await;
                tokio::spawn(async move {
                    PluginManager::update_loop(plugin).await;
                });
            }
        }
        .boxed()
    }
}
