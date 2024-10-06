use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use host_plug::HostPlugin;
use plugin::{Plugin, PluginRef};
use thiserror::Error;

pub mod cacher;
pub mod host_plug;
pub mod plugin;

// --- Internal Types ---

pub(crate) type ArRw<T> = Arc<RwLock<T>>;
// pub(crate) type WeRw<T> = Weak<RwLock<T>>;

// /// Environment to pass to host functions
// pub(crate) type RuntimeEnv = (Universe, Cacher);

pub use bugic_share as param;

// --- Universe ---

/// Stores plugins
#[derive(Clone)]
pub struct Universe(ArRw<UniverseInner>);

// /// Weak reference to Universe
// #[derive(Clone)]
// pub struct UniverseWeak(WeRw<UniverseInner>);

/// Inner data of Universe
struct UniverseInner {
    plugins: HashMap<PluginId, Arc<Plugin>>,
    next_id: PluginId,
}

/// Plugin Reference ID
pub(crate) type PluginId = u32;

/// Plugin Function Symbol
pub(crate) type PluginSymbol = String;

impl Universe {
    /// Create a new Universe
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(UniverseInner {
            plugins: HashMap::new(),
            next_id: 0,
        })))
    }

    /// Add a plugin to the Universe
    pub fn add_plugin(&self, plugin: Plugin) -> Result<PluginRef, BugiError> {
        let mut inner = self.0.write().unwrap();

        // Check ID
        for (_, p) in inner.plugins.iter() {
            if p.get_str_id() == plugin.get_str_id() {
                return Err(BugiError::PluginIdExists(plugin.get_str_id().clone()));
            }
        }

        let id = inner.next_id;
        inner.next_id += 1;

        let plugin = Arc::new(plugin);

        inner.plugins.insert(id, Arc::clone(&plugin));
        Ok(PluginRef::new(Arc::downgrade(&plugin)))
    }

    /// Add a host plugin to the Universe
    pub fn add_host_plugin(
        &self,
        str_id: String,
        host: HostPlugin,
    ) -> Result<PluginRef, BugiError> {
        let plugin = Plugin::make_host(str_id, host);
        self.add_plugin(plugin)
    }
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

// --- Error ---

#[derive(Error, Debug)]
pub enum BugiError {
    #[error("cannot serialize: {0}")]
    CannotSerialize(#[from] param::SerializeError),

    #[error("the plugin ID already exists: {0}")]
    PluginIdExists(String),

    #[error("plugin is dropped")]
    PluginDropped,

    #[error("plugin call error: {0}")]
    PluginCallError(String),

    #[error("plugin abi error: {0}, {1}")]
    PluginAbiError(u8, u8),
}
