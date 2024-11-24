use {
    crate::db::Database,
    futures::StreamExt,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    types::{
        api::CompressedEvent,
        available_plugins::AvailablePlugins,
        external::{chrono::Utc, serde_json},
    },
    url::Url,
};

#[derive(Serialize, Deserialize)]
struct Error {
    plugin: Option<AvailablePlugins>,
    error: String,
}

pub struct Plugin {
    plugin_data: crate::plugin::PluginData,
}

impl crate::plugin::PluginTrait for Plugin {
    async fn new(data: crate::plugin::PluginData) -> Self
    where
        Self: Sized,
    {
        Plugin { plugin_data: data }
    }

    fn get_compressed_events(
        &self,
        query_range: &types::timing::TimeRange,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<Output = types::api::APIResult<Vec<types::api::CompressedEvent>>>
                + Send,
        >,
    > {
        let filter = Database::generate_range_filter(query_range);
        let plg_filter = Database::generate_find_plugin_filter(AvailablePlugins::error);
        let filter = Database::combine_documents(filter, plg_filter);
        let database = self.plugin_data.database.clone();
        Box::pin(async move {
            let mut cursor = database.get_events::<Error>().find(filter, None).await?;
            let mut result = Vec::new();
            while let Some(v) = cursor.next().await {
                let t = v?;
                result.push(CompressedEvent {
                    title: t
                        .event
                        .plugin
                        .clone()
                        .map_or("Error".to_string(), |v| v.to_string()),
                    time: t.timing,
                    data: serde_json::to_value(t.event).unwrap(),
                })
            }

            Ok(result)
        })
    }

    fn get_type() -> types::available_plugins::AvailablePlugins
    where
        Self: Sized,
    {
        types::available_plugins::AvailablePlugins::error
    }
}

pub fn error(
    database: Arc<Database>,
    error: &impl std::error::Error,
    plugin: Option<AvailablePlugins>,
    error_url: &Option<Url>,
) {
    error_string(database, format!("{}", error), plugin, error_url)
}

pub fn error_string(
    database: Arc<Database>,
    error: String,
    plugin: Option<AvailablePlugins>,
    error_url: &Option<Url>,
) {
    if let Some(ref url) = error_url {
        let mut url = url.clone();
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("error", &error);
        if let Some(v) = plugin.clone() {
            pairs.append_pair("plugin", &v.to_string());
        }
        drop(pairs);
        let url = url.clone();

        tokio::spawn(async move {
            let res = types::external::reqwest::get(url).await;
            if let Err(e) = res {
                println!("Unable to perform get request on error occurrence: {}", e);
            }
        });
    }

    tokio::spawn(async move {
        let now = Utc::now();
        let res = database
            .register_single_event(&crate::db::Event {
                timing: types::timing::Timing::Instant(now),
                id: now.timestamp_millis().to_string(),
                plugin: AvailablePlugins::error,
                event: Error {
                    plugin,
                    error: error.clone(),
                },
            })
            .await;

        if let Err(e) = res {
            println!("Was unable to report error to database: {e}. Original error: \n{error}")
        }
    });
}
