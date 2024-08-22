#![feature(let_chains)]

pub mod api;
mod error;
pub mod events_display;
pub mod plugin_manager;

use plugin_manager::Plugin;
use plugin_manager::PluginData;
use std::collections::HashMap;
use types::api::AvailablePlugins;

include!(concat!(env!("OUT_DIR"), "/plugins.rs"));
