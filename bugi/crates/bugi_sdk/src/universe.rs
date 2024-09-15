use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use rand::{rngs::SmallRng, RngCore as _, SeedableRng as _};
use wasmtime::{Engine, Module, Store};

use anyhow::{Context, Result};

use crate::module::{empty_import, get_desc, PluginID, PluginModule, PluginRef};

const RAND_SEED: u64 = 20240911;

pub struct PluginUniverse {
    inner: Arc<RwLock<PluginUniverseInner>>,
}

pub(crate) struct PluginUniverseInner {
    str_id: HashMap<String, PluginID>,
    module: HashMap<PluginID, PluginModule>,
    rng: SmallRng,
    engine: Engine,
}

impl PluginUniverse {
    pub fn new() -> Self {
        PluginUniverse {
            inner: Arc::new(RwLock::new(PluginUniverseInner {
                rng: SmallRng::seed_from_u64(RAND_SEED),
                module: HashMap::default(),
                engine: Engine::default(),
                str_id: HashMap::default(),
            })),
        }
    }

    pub fn add_module_from_file(&mut self, path: impl AsRef<Path>) -> Result<PluginRef> {
        let mut data = self.inner.write().unwrap();
        let id = data.rng.next_u32();
        let module = Module::from_file(&data.engine, path)?;

        let desc = get_desc(&data.engine, &module)?;

        data.module.insert(
            id,
            PluginModule {
                id,
                module,
                str_id: desc.string_id,
            },
        );

        Ok(PluginRef { id })
    }

    pub fn add_module_from_binary(&mut self, bin: &[u8]) -> Result<PluginRef> {
        let mut data = self.inner.write().unwrap();
        let id = data.rng.next_u32();
        let module = Module::from_binary(&data.engine, bin)?;
        let desc = get_desc(&data.engine, &module)?;

        data.module.insert(
            id,
            PluginModule {
                id,
                module,
                str_id: desc.string_id,
            },
        );

        Ok(PluginRef { id })
    }
}

impl Default for PluginUniverse {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PluginUniverse {
    fn clone(&self) -> Self {
        PluginUniverse {
            inner: Arc::clone(&self.inner),
        }
    }
}
