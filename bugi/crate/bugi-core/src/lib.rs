use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub use bugi_share::*;

type ResultType = Result<(Vec<u8>, Option<Box<dyn Any>>), BugiError>;
pub trait PluginSystem: Send + Sync {
    /// call a plugin function
    /// if cache is unit value, it means no cache
    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi: u8,
        get_global: Option<CachePloxy>,
    ) -> ResultType;
}

#[derive(thiserror::Error, Debug)]
pub enum BugiError {
    #[error("cannot serialize: {0}")]
    CannotSerialize(#[from] bugi_share::SerializeError),

    #[error("the plugin ID already exists: {0}")]
    PluginIdExists(String),

    #[error("plugin is dropped")]
    PluginDropped,

    #[error("plugin call error: {0}")]
    PluginCallError(String),

    #[error("plugin abi error: expected abi = {0}")]
    PluginAbiError(u8),
}

/// Plugin Reference ID
pub type PluginId = u32;

#[derive(Default, Clone)]
pub struct Cacher(Arc<RwLock<CacherInner>>);

#[derive(Default)]
struct CacherInner {
    cache: HashMap<PluginId, Box<dyn Any>>,
    cache_global: HashMap<String, Box<dyn Any>>,
}

impl Cacher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop(&self, id: PluginId) -> Option<Box<dyn Any>> {
        let mut inner = self.0.write().unwrap();
        inner.cache.remove(&id)
    }

    pub fn push(&self, id: PluginId, data: Box<dyn Any>) {
        let mut inner = self.0.write().unwrap();
        inner.cache.insert(id, data);
    }

    pub fn pop_global(&self, id: &str) -> Option<Box<dyn Any>> {
        let mut inner = self.0.write().unwrap();
        inner.cache_global.remove(id)
    }

    pub fn push_global(&self, id: &str, data: Box<dyn Any>) {
        let mut inner = self.0.write().unwrap();
        inner.cache_global.insert(id.to_string(), data);
    }

    pub fn get_gcache(&self, plugin_cache: Option<Box<dyn Any>>) -> CachePloxy {
        let a = (*self).clone();
        let b = (*self).clone();

        CachePloxy::new(
            move |id| a.pop_global(id).unwrap(),
            move |id, data| b.push_global(id, data),
            plugin_cache,
        )
    }
}

pub struct CachePloxy {
    #[allow(clippy::type_complexity)]
    get: Box<dyn Fn(&str) -> Box<dyn Any>>,
    #[allow(clippy::type_complexity)]
    set: Box<dyn Fn(&str, Box<dyn Any>)>,
    pub plugin_cache: Option<Box<dyn Any>>,
}

impl CachePloxy {
    pub fn new(
        get: impl Fn(&str) -> Box<dyn Any> + 'static,
        set: impl Fn(&str, Box<dyn Any>) + 'static,
        plugin_cache: Option<Box<dyn Any>>,
    ) -> Self {
        Self {
            get: Box::new(get),
            set: Box::new(set),
            plugin_cache,
        }
    }

    pub fn get(&self, id: &str) -> Box<dyn Any> {
        (self.get)(id)
    }

    pub fn set(&self, id: &str, data: Box<dyn Any>) {
        (self.set)(id, data)
    }
}
