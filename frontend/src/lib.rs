#![feature(let_chains)]

pub mod api;
mod error;
pub mod events_display;
pub mod plugin_manager;
pub mod wrappers;

use {
    plugin_manager::{Plugin, PluginData},
    std::collections::HashMap,
    types::api::AvailablePlugins,
};

include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
