use std::fmt;

use leptos::{component, view, Children, IntoView};

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Acc1,
    Acc2,
    Light,
    Dark,
    Custom(String, String, String),
}

impl Style {
    pub fn light(&self) -> &str {
        match self {
            Style::Acc1 => "var(--accentColor1Light)",
            Style::Acc2 => "var(--accentColor2Light)",
            Style::Light => "var(--lightColor)",
            Style::Dark => "var(--darkColor)",
            Style::Custom(_, light_color, _) => light_color,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Style::Light => "var(--darkColor)",
            Style::Custom(_, _, text_color) => text_color,
            _ => "var(--lightColor)",
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Acc1 => {
                write!(f, "var(--accentColor1)")
            }
            Style::Acc2 => {
                write!(f, "var(--accentColor2)")
            }
            Style::Light => {
                write!(f, "var(--lighterColor)")
            }
            Style::Dark => {
                write!(f, "var(--darkColorLight)")
            }
            Style::Custom(dark_color, _, _) => write!(f, "{}", dark_color),
        }
    }
}

#[component]
pub fn StyledView(children: Children) -> impl IntoView {
    view! {
        <div class="view" style="display: flex;flex-direction: column;width: 100%;height: 100%;">
            {children()}
        </div>
    }
}
