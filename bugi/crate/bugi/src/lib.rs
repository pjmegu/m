use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};

use plugin::{Plugin, PluginRef};

mod r#override;
mod plugin;

// --- Re-exports ---

pub use bugi_core::*;

#[allow(unused_imports)]
pub use bugi_share::*;

#[allow(unused_imports)]
pub use plugin::*;

#[allow(unused_imports)]
pub use r#override::*;

#[cfg(feature = "plug-host")]
pub use bugi_host::*;

#[cfg(feature = "plug-wasm")]
pub use bugi_wasm::*;

// --- Universe ---

/// Stores plugins
#[derive(Clone)]
pub struct Universe(Arc<RwLock<UniverseInner>>);

/// Stores plugins
#[derive(Clone)]
pub(crate) struct UniverseWeak(Weak<RwLock<UniverseInner>>);

/// Inner data of Universe
struct UniverseInner {
    plugins: HashMap<PluginId, Arc<Plugin>>,
    str_ids: HashMap<String, PluginId>,
    next_id: PluginId,
}

impl Universe {
    /// Create a new Universe
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(UniverseInner {
            plugins: HashMap::new(),
            str_ids: HashMap::new(),
            next_id: 0,
        })))
    }

    /// Add a plugin to the Universe
    pub fn add_plugin_raw(&self, plugin: Plugin) -> Result<PluginRef, BugiError> {
        let mut inner = self.0.write().unwrap();

        // Check ID
        // TODO: This is O(n) and can be optimized to use a HashSet
        // Modify this if performance becomes an issue
        for (s, _) in inner.str_ids.iter() {
            if s == plugin.get_str_id() {
                return Err(BugiError::PluginIdExists(plugin.get_str_id().to_string()));
            }
        }

        let id = inner.next_id;
        inner.next_id += 1;

        let plugin = Arc::new(plugin);

        inner.plugins.insert(id, Arc::clone(&plugin));
        inner.str_ids.insert(plugin.get_str_id().to_string(), id);
        Ok(PluginRef::new(
            Arc::downgrade(&plugin),
            id,
            UniverseWeak(Arc::downgrade(&self.0)),
        ))
    }

    /// add plugin with PluginSystem
    pub fn add_plugin(
        &self,
        str_id: &str,
        detail: impl bugi_core::PluginSystem + 'static,
    ) -> Result<PluginRef, BugiError> {
        self.add_plugin_raw(Plugin::new(str_id, detail))
    }

    pub(crate) fn call_raw(
        &self,
        str_id: &str,
        symbol: &str,
        arg: &[u8],
        abi: u8,
        ploxy: EnvPloxy,
    ) -> Result<Vec<u8>, BugiError> {
        let inner = self.0.read().unwrap();
        let id = inner
            .str_ids
            .get(str_id)
            .ok_or(BugiError::PluginNotFound(str_id.to_string()))?;
        let plugin = inner.plugins.get(id).unwrap();
        plugin.call_raw(symbol, arg, abi, ploxy)
    }

    pub(crate) fn call_raw_id(
        &self,
        id: PluginId,
        symbol: &str,
        arg: &[u8],
        abi: u8,
        ploxy: EnvPloxy,
    ) -> Result<Vec<u8>, BugiError> {
        let inner = self.0.read().unwrap();
        let plugin = inner.plugins.get(&id).unwrap();
        plugin.call_raw(symbol, arg, abi, ploxy)
    }
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

impl UniverseWeak {
    /// Upgrade to a strong reference
    pub fn upgrade(&self) -> Option<Universe> {
        self.0.upgrade().map(Universe)
    }
}
