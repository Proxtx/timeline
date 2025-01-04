use {
    server_api::{
        external::{
            futures::{self, FutureExt, StreamExt},
            tokio::{self, sync::RwLock},
            types::{api::APIResult, available_plugins::AvailablePlugins, timing::TimeRange},
        },
        plugin::PluginTrait,
    },
    std::{collections::HashMap, panic::AssertUnwindSafe, pin::Pin, sync::Arc, time::Duration},
};

type ThreadedPlugin = Arc<RwLock<Box<dyn PluginTrait>>>;
type PluginsMap = HashMap<AvailablePlugins, ThreadedPlugin>;
pub struct PluginManager {
    plugins: PluginsMap,
    #[allow(dead_code)]
    panic_hook: Arc<dyn Fn(String, AvailablePlugins) + Send + Sync>,
}

impl PluginManager {
    pub fn new(
        plugins: HashMap<AvailablePlugins, Box<dyn PluginTrait>>,
        panic_hook: Arc<dyn Fn(String, AvailablePlugins) + Send + Sync>,
    ) -> Self {
        let plugins: PluginsMap = plugins
            .into_iter()
            .map(|(key, value)| (key, Arc::new(RwLock::new(value))))
            .collect();
        for (av_plugin, plg) in plugins.iter() {
            let plg = plg.clone();
            let plg_mut = plg.clone();
            let av_plugin = av_plugin.clone();
            let av_plugin_2 = av_plugin.clone();
            let panic_hook = panic_hook.clone();
            let panic_hook_2 = panic_hook.clone();
            tokio::spawn(async move {
                PluginManager::update_loop_mut(av_plugin, plg_mut, panic_hook).await;
            });
            tokio::spawn(async move {
                PluginManager::update_loop(av_plugin_2.clone(), plg, panic_hook_2.clone()).await;
            });
        }
        PluginManager {
            plugins,
            panic_hook,
        }
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

    pub fn update_loop_mut(
        av_plugin: AvailablePlugins,
        plugin: ThreadedPlugin,
        panic_hook: Arc<dyn Fn(String, AvailablePlugins) + Send + Sync>,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let mutable_plugin;
            {
                let mut mut_plg = plugin.write().await;
                let fut = mut_plg.request_loop_mut();
                mutable_plugin = AssertUnwindSafe(fut).catch_unwind().await;
            }
            match mutable_plugin {
                Ok(v) => {
                    if let Some(v) = v {
                        tokio::time::sleep(v.to_std().unwrap()).await;
                        tokio::spawn(async move {
                            PluginManager::update_loop_mut(av_plugin, plugin, panic_hook).await;
                        });
                    }
                }
                Err(e) => {
                    panic_hook(
                        format!(
                            "{} panicked during a mut update loop: {:?}",
                            av_plugin.clone(),
                            e
                        ),
                        av_plugin.clone(),
                    );
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    tokio::spawn(async move {
                        PluginManager::update_loop_mut(av_plugin, plugin, panic_hook).await;
                    });
                }
            }
        }
        .boxed()
    }

    pub fn update_loop(
        av_plugin: AvailablePlugins,
        plugin: ThreadedPlugin,
        panic_hook: Arc<dyn Fn(String, AvailablePlugins) + Send + Sync>,
    ) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> {
        async move {
            let immutable_plugin;
            {
                let mut_plg = plugin.read().await;
                let fut = mut_plg.request_loop();
                immutable_plugin = AssertUnwindSafe(fut).catch_unwind().await;
            }
            match immutable_plugin {
                Ok(v) => {
                    if let Some(v) = v {
                        tokio::time::sleep(v.to_std().unwrap()).await;
                        tokio::spawn(async move {
                            PluginManager::update_loop(av_plugin, plugin, panic_hook).await;
                        });
                    }
                }
                Err(e) => {
                    panic_hook(
                        format!(
                            "{} panicked during a update loop: {:?}",
                            av_plugin.clone(),
                            e
                        ),
                        av_plugin.clone(),
                    );
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    tokio::spawn(async move {
                        PluginManager::update_loop(av_plugin, plugin, panic_hook).await;
                    });
                }
            }
        }
        .boxed()
    }

    pub fn get_plugin(&self, plugin: &AvailablePlugins) -> &ThreadedPlugin {
        self.plugins.get(plugin).unwrap()
    }
}
