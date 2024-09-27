use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock, Weak},
};

use rand::{rngs::SmallRng, RngCore as _, SeedableRng as _};
use wasmtime::{Engine, Module};

use anyhow::Result;

use crate::module::{get_desc, PluginID, PluginModule, PluginRef};

const RAND_SEED: u64 = 20240911;

pub struct PluginUniverse(Arc<RwLock<PluginUniverseInner>>);
pub(crate) struct PluginUniverseWeak(Weak<RwLock<PluginUniverseInner>>);

struct PluginUniverseInner {
    pub(crate) str_id: HashMap<String, PluginID>,
    pub(crate) module: HashMap<PluginID, Arc<RwLock<PluginModule>>>,
    pub(crate) rng: SmallRng,
    pub(crate) engine: Engine,
}

impl PluginUniverse {
    pub fn new() -> Self {
        PluginUniverse(Arc::new(RwLock::new(PluginUniverseInner {
            rng: SmallRng::seed_from_u64(RAND_SEED),
            module: HashMap::default(),
            engine: Engine::default(),
            str_id: HashMap::default(),
        })))
    }

    pub fn add_module_from_file(&mut self, path: impl AsRef<Path>) -> Result<PluginRef> {
        let mut data = self.0.write().unwrap();
        let id = data.rng.next_u32();
        let module = Module::from_file(&data.engine, path)?;

        let desc = get_desc(&data.engine, &module)?;

        let module = Arc::new(RwLock::new(PluginModule {
            id,
            module,
            // str_id: desc.string_id.clone(),
            // univ: self.get_weak(),
        }));

        data.module.insert(id, Arc::clone(&module));

        data.str_id.insert(desc.string_id, id);

        Ok(PluginRef {
            id,
            univ: self.get_weak(),
            module: Arc::downgrade(&module),
        })
    }

    pub fn add_module_from_binary(&mut self, bin: &[u8]) -> Result<PluginRef> {
        let mut data = self.0.write().unwrap();
        let id = data.rng.next_u32();
        let module = Module::from_binary(&data.engine, bin)?;
        let desc = get_desc(&data.engine, &module)?;

        let module = Arc::new(RwLock::new(PluginModule {
            id,
            module,
            // str_id: desc.string_id.clone(),
            // univ: self.get_weak(),
        }));

        data.module.insert(id, Arc::clone(&module));

        data.str_id.insert(desc.string_id, id);

        Ok(PluginRef {
            id,
            univ: self.get_weak(),
            module: Arc::downgrade(&module),
        })
    }

    pub(crate) fn get_engine(&self) -> Engine {
        self.0.read().unwrap().engine.clone()
    }

    pub(crate) fn get_module(&self, id: PluginID) -> Option<Arc<RwLock<PluginModule>>> {
        self.0
            .read()
            .unwrap()
            .module
            .get(&id)
            .map(Arc::clone)
    }

    pub(crate) fn get_str_id(&self, id: &str) -> Option<PluginID> {
        self.0.read().unwrap().str_id.get(id).copied()
    }

    pub(crate) fn get_weak(&self) -> PluginUniverseWeak {
        PluginUniverseWeak(Arc::downgrade(&self.0))
    }
}

impl Default for PluginUniverse {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PluginUniverse {
    fn clone(&self) -> Self {
        PluginUniverse(Arc::clone(&self.0))
    }
}

impl PluginUniverseWeak {
    pub fn upgrade(&self) -> Option<PluginUniverse> {
        self.0.upgrade().map(PluginUniverse)
    }
}