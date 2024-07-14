use {
    crate::{AvailablePlugins, Plugin}, chrono::Duration, futures::{FutureExt, StreamExt}, std::{collections::HashMap, pin::Pin, sync::Arc}, tokio::sync::RwLock, types::{api::{APIResult, CompressedEvent}, timing::TimeRange}
};

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

    pub async fn get_compress_events(
        &self,
        time_range: &TimeRange,
    ) -> APIResult<HashMap<AvailablePlugins, Vec<crate::CompressedEvent>>> {
        let mut futures = futures::stream::FuturesUnordered::new();
        for (name, plugin) in self.plugins.iter() {
            futures.push(async move {
                (
                    name.clone(),
                    plugin.read().await.get_compressed_events(time_range).await,
                )
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

    pub async fn latest_event(&self, span: Duration) -> APIResult<(AvailablePlugins, CompressedEvent)> {
        
    }

    pub fn update_loop_mut(
        plugin: ThreadedPlugin,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let mutable_plugin;
            {
                let mut mut_plg = plugin.write().await;
                let fut = mut_plg.request_loop_mut();
                mutable_plugin = fut.await;
            }
            if let Some(v) = mutable_plugin {
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
            let immutable_plugin;
            {
                let mut_plg = plugin.read().await;
                let fut = mut_plg.request_loop();
                immutable_plugin = fut.await;
            }
            if let Some(v) = immutable_plugin {
                tokio::time::sleep(v.to_std().unwrap()).await;
                tokio::spawn(async move {
                    PluginManager::update_loop(plugin).await;
                });
            }
        }
        .boxed()
    }

    pub fn get_plugin(&self, plugin: &AvailablePlugins) -> &ThreadedPlugin {
        self.plugins.get(plugin).unwrap()
    }
}
