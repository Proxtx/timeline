use std::fmt;

use serde::{Deserialize, Serialize};

#[link_proc_macro::generate_available_plugins]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum AvailablePlugins {
    error,
}

impl fmt::Display for AvailablePlugins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
