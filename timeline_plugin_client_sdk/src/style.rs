//! Style mirrors the old `client_api::Style`. Wire format matches
//! [`timeline_plugin_sdk::Style`] (server-side manifest).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Style {
    Acc1,
    Acc2,
    Light,
    Dark,
    /// An explicit CSS color string (e.g. `"rgb(20,40,80)"`). The frontend
    /// derives matching light/text colors from it.
    Custom(String),
}

impl Default for Style {
    fn default() -> Self {
        Style::Acc1
    }
}

impl Style {
    /// CSS var to use for the dark (background) color.
    pub fn bg_var(&self) -> &'static str {
        match self {
            Style::Acc1 => "var(--accentColor1)",
            Style::Acc2 => "var(--accentColor2)",
            Style::Light => "var(--lightColor)",
            Style::Dark => "var(--darkColor)",
            Style::Custom(_) => "var(--pluginBg)",
        }
    }

    pub fn fg_var(&self) -> &'static str {
        match self {
            Style::Light => "var(--darkColor)",
            _ => "var(--lightColor)",
        }
    }

    /// Literal color for [`Style::Custom`], else `None`.
    pub fn custom_color(&self) -> Option<&str> {
        match self {
            Style::Custom(c) => Some(c),
            _ => None,
        }
    }
}
