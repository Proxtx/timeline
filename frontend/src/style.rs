//! Per-plugin color styling. Mirrors the server manifest shape and matches
//! the CSS variables in `style.css`.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Style {
    Acc1,
    Acc2,
    Light,
    Dark,
    Custom(String),
}

impl Default for Style {
    fn default() -> Self {
        Style::Acc1
    }
}

impl Style {
    /// Background (dark) color.
    pub fn bg(&self) -> String {
        match self {
            Style::Acc1 => "var(--accentColor1)".into(),
            Style::Acc2 => "var(--accentColor2)".into(),
            Style::Light => "var(--lighterColor)".into(),
            Style::Dark => "var(--darkColorLight)".into(),
            Style::Custom(c) => c.clone(),
        }
    }

    /// Lighter tint used for event row backgrounds.
    pub fn light(&self) -> String {
        match self {
            Style::Acc1 => "var(--accentColor1Light)".into(),
            Style::Acc2 => "var(--accentColor2Light)".into(),
            Style::Light => "var(--lightColor)".into(),
            Style::Dark => "var(--darkColor)".into(),
            Style::Custom(c) => c.clone(),
        }
    }

    /// Foreground text color.
    pub fn text(&self) -> &'static str {
        match self {
            Style::Light => "var(--darkColor)",
            _ => "var(--lightColor)",
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bg())
    }
}
