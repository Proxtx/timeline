//! Plugin manifest exposed at `GET /manifest`. The main timeline server
//! aggregates all plugin manifests into `/api/plugins` for the frontend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub display_name: String,
    pub style: Style,
    /// Optional: path served by the plugin (or by the main server) that
    /// resolves to the plugin's icon image. Unset → main frontend falls
    /// back to a default icon.
    #[serde(default)]
    pub icon: Option<String>,
    /// Relative path (inside `<data_dir>/plugin_web/<name>/`) to the plugin's
    /// wasm/js entrypoint, set at plugin build time. Clients import this to
    /// mount the plugin UI into a shadow root.
    #[serde(default)]
    pub web_entry: Option<String>,
}

/// Mirrors the old `client_api::style::Style` enum. Preserves the CSS-variable
/// mapping so the main frontend can keep using `var(--accentColor1)` etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Style {
    Acc1,
    Acc2,
    Light,
    Dark,
    /// RGB triplet of the dark tone. The frontend derives light/text from it.
    Custom(String),
}

impl Default for Style {
    fn default() -> Self {
        Style::Acc1
    }
}
