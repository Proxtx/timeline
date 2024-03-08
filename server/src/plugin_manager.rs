use futures::FutureExt;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::pin;
use tokio::sync::RwLock;

use crate::Plugin;

type ThreadedPlugin = Arc<RwLock<Box<dyn Plugin>>>;
type PluginsMap = HashMap<String, ThreadedPlugin>;
pub struct PluginManager {
    plugins: PluginsMap,
}

impl PluginManager {
    pub fn new(plugins: HashMap<String, Box<dyn Plugin>>) -> Self {
        let plugins: PluginsMap = plugins
            .into_iter()
            .map(|(key, value)| (key, Arc::new(RwLock::new(value))))
            .collect();
        for (_, plg) in plugins.iter() {
            let plg = plg.clone();
            let plg_mut = plg.clone();
            tokio::spawn(async move {
                PluginManager::update_loop(plg).await;
            });
            tokio::spawn(async move {
                PluginManager::update_loop_mut(plg_mut).await;
            });
        }
        PluginManager { plugins }
    }

    pub fn update_loop_mut(
        plugin: ThreadedPlugin,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let lptm;
            {
                let mut mut_plg = plugin.write().await;
                let fut = mut_plg.request_loop_mut();
                pin!(fut);
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

    pub fn update_loop(
        plugin: ThreadedPlugin,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let lptm;
            {
                let mut_plg = plugin.read().await;
                let fut = mut_plg.request_loop();
                pin!(fut);
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
