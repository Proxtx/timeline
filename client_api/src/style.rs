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
