//! Async error reporting. Fire-and-forget POST to the configured
//! `error_report_url` (if any) plus a local log line. Matches the semantics
//! of the old `server_api::error::error_string` helper.

use std::sync::Arc;

use reqwest::Client;
use url::Url;

#[derive(Clone)]
pub struct ErrorReporter {
    inner: Arc<Inner>,
}

struct Inner {
    plugin_name: String,
    url: Option<Url>,
    client: Client,
}

impl ErrorReporter {
    pub fn new(plugin_name: impl Into<String>, url: Option<Url>) -> Self {
        Self {
            inner: Arc::new(Inner {
                plugin_name: plugin_name.into(),
                url,
                client: Client::new(),
            }),
        }
    }

    pub fn report(&self, message: impl Into<String>) {
        let message = message.into();
        let me = self.inner.clone();
        tokio::spawn(async move {
            tracing::error!(plugin = %me.plugin_name, "{}", message);
            if let Some(url) = &me.url {
                let mut url = url.clone();
                url.query_pairs_mut()
                    .append_pair("plugin", &me.plugin_name)
                    .append_pair("error", &message);
                if let Err(e) = me.client.get(url).send().await {
                    tracing::warn!(plugin = %me.plugin_name, "error webhook failed: {}", e);
                }
            }
        });
    }

    pub fn report_err(&self, e: &(impl std::error::Error + ?Sized)) {
        self.report(e.to_string());
    }
}
