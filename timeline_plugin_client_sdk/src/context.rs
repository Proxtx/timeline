//! Context injected into every plugin render call.

use serde::{Deserialize, Serialize};

use types::api::CompressedEvent;

use crate::style::Style;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// Rendering a single event card.
    Event,
    /// Rendering a day-level summary of this plugin's events.
    Overview,
    /// Rendering as a standalone takeover (slide-over panel, etc.).
    Standalone,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Event
    }
}

/// Everything a plugin UI needs to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Stable plugin name (matches server `plugins.toml` key).
    pub plugin_name: String,
    /// Proxy prefix to talk back to the plugin: `/api/plugin/<name>`.
    pub api_base: String,
    /// The event being rendered (present for `Event` mode; for other modes
    /// plugins pass a sentinel / construct their own view data).
    pub event: CompressedEvent,
    pub style: Style,
    pub mode: RenderMode,
}
