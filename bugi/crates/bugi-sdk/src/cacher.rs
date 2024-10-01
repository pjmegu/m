use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use wasmtime::Store;

use crate::{
    module::{PluginID, PluginInstance},
    universe::PluginUniverse,
};

/// Cache instances of plugins.  
/// It is recommended that this structure exist per thread, but it can be shared among threads.  
/// When calling a plugin, it may or may not be included in the arguments,
/// but the cacher is created inside the function and discarded at the end of the function call.
#[derive(Clone)]
pub struct PluginCacher(Arc<RwLock<PluginCacherInner>>);

pub(crate) struct PluginCacherInner {
    store: Arc<RwLock<Store<OnceLock<PluginCacher>>>>,
    ins: HashMap<PluginID, Arc<PluginInstance>>,
    univ: PluginUniverse,
}

impl PluginCacher {
    pub(crate) fn new(univ: &PluginUniverse) -> Self {
        let cacher = Arc::new(RwLock::new(PluginCacherInner {
            store: Arc::new(RwLock::new(Store::new(&univ.get_engine(), OnceLock::new()))),
            ins: HashMap::new(),
            univ: PluginUniverse::clone(univ),
        }));
        let c_cacher = Arc::clone(&cacher);
        cacher
            .write()
            .unwrap()
            .store
            .write()
            .unwrap()
            .data_mut()
            .get_or_init(|| Self(c_cacher));
        Self(cacher)
    }

    pub(crate) fn add_ins(&self, id: PluginID, ins: wasmtime::Instance) {
        let mut data = self.0.write().unwrap();
        data.ins.insert(id, Arc::new(PluginInstance { ins }));
    }

    pub(crate) fn get_ins(&self, id: PluginID) -> Option<Arc<PluginInstance>> {
        let data = self.0.read().unwrap();
        data.ins.get(&id).map(Arc::clone)
    }

    #[inline]
    pub(crate) fn get_or_add_ins(
        &self,
        id: PluginID,
        ins: impl FnOnce() -> wasmtime::Instance,
    ) -> Arc<PluginInstance> {
        let mut data = self.0.write().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = data.ins.entry(id) {
            let ins = Arc::new(PluginInstance { ins: ins() });
            e.insert(Arc::clone(&ins));
            ins
        } else {
            Arc::clone(data.ins.get(&id).unwrap())
        }
    }

    pub(crate) fn get_store(&self) -> Arc<RwLock<Store<OnceLock<PluginCacher>>>> {
        Arc::clone(&self.0.read().unwrap().store)
    }

    pub(crate) fn get_univ(&self) -> PluginUniverse {
        self.0.read().unwrap().univ.clone()
    }
}
