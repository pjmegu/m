use std::{any::Any, collections::HashMap, sync::{Arc, RwLock}};

use bugi_core::PluginId;

#[derive(Default)]
pub struct Cacher(Arc<RwLock<CacherInner>>);

#[derive(Default)]
struct CacherInner {
    cache: HashMap<PluginId, Box<dyn Any>>
}

impl Cacher {
    pub fn new() -> Self{
        Self::default()
    }

    pub(crate) fn pop(&self, id: PluginId) -> Option<Box<dyn Any>> {
        let mut inner = self.0.write().unwrap();
        inner.cache.remove(&id)
    }

    pub(crate) fn push(&self, id: PluginId, data: Box<dyn Any>) {
        let mut inner = self.0.write().unwrap();
        inner.cache.insert(id, data);
    }
}