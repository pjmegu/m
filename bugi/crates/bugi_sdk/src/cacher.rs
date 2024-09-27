use std::{
    cell::OnceCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use wasmtime::Store;

use crate::{
    module::{PluginID, PluginInstance},
    spec_type::FuncDesc,
    universe::PluginUniverse,
};

pub struct PluginCacher(Arc<RwLock<PluginCacherInner>>);

pub(crate) struct PluginCacherInner {
    symbol_desc: HashMap<(PluginID, String), FuncDesc>,
    store: Arc<RwLock<Store<OnceCell<PluginCacher>>>>,
    ins: HashMap<PluginID, Arc<PluginInstance>>,
    max_drop_time: Option<u32>,
    now_drop_time: u32,
    univ: PluginUniverse,
}

impl PluginCacher {
    pub(crate) fn new(univ: &PluginUniverse, max_drop_time: Option<u32>) -> Self {
        let cacher = Arc::new(RwLock::new(PluginCacherInner {
            symbol_desc: HashMap::new(),
            store: Arc::new(RwLock::new(Store::new(&univ.get_engine(), OnceCell::new()))),
            ins: HashMap::new(),
            max_drop_time,
            now_drop_time: 0,
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
    pub(crate) fn get_or_add_ins(&self, id: PluginID, ins: impl FnOnce() -> wasmtime::Instance) -> Arc<PluginInstance>{
        let mut data = self.0.write().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = data.ins.entry(id) {
            let ins = Arc::new(PluginInstance { ins: ins() });
            e.insert(Arc::clone(&ins));
            ins
        } else {
            Arc::clone(data.ins.get(&id).unwrap())
        }
    }

    pub(crate) fn get_store(&self) -> Arc<RwLock<Store<OnceCell<PluginCacher>>>> {
        Arc::clone(&self.0.read().unwrap().store)
    }

    pub(crate) fn inc_drop_time(&self) {
        let mut data = self.0.write().unwrap();
        data.now_drop_time += 1;
    }

    pub(crate) fn check_drop_time(&self) -> bool {
        let data = self.0.read().unwrap();
        if let Some(max) = data.max_drop_time {
            data.now_drop_time >= max
        } else {
            false
        }
    }

    pub(crate) fn get_univ(&self) -> PluginUniverse {
        self.0.read().unwrap().univ.clone()
    }
}
