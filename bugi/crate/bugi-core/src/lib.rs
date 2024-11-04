#![allow(clippy::type_complexity)]

use std::{
    any::Any,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

pub use bugi_share::*;

pub trait PluginSystem: Send + Sync {
    fn str_id(&self) -> String;
    /// call a plugin function
    /// if cache is unit value, it means no cache
    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi: u64,
        ploxy: EnvPloxy,
    ) -> Result<Vec<u8>, BugiError>;
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
    PluginAbiError(u64),

    #[error("plugin not found: {0}")]
    PluginNotFound(String),

    #[error("plugin universe dropped")]
    PluginUniverseDropped,
}

/// Plugin Reference ID
pub type PluginId = u32;

pub type CacheData = Box<dyn Any + Send + Sync>;

#[derive(Default, Clone)]
pub struct Cacher(Arc<Mutex<CacherInner>>);

#[derive(Default)]
struct CacherInner {
    cache: HashMap<PluginId, CacheData>,
    cache_global: HashMap<String, CacheData>,
}

impl Cacher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop(&self, id: PluginId) -> Option<CacheData> {
        self.0.deref().lock().unwrap().cache.remove(&id)
    }

    pub fn push(&self, id: PluginId, data: CacheData) {
        self.0.deref().lock().unwrap().cache.insert(id, data);
    }

    pub fn pop_global(&self, id: &str) -> Option<CacheData> {
        self.0.deref().lock().unwrap().cache_global.remove(id)
    }

    pub fn push_global(&self, id: &str, data: CacheData) {
        self.0
            .deref()
            .lock()
            .unwrap()
            .cache_global
            .insert(id.to_string(), data);
    }
}

#[derive(Clone)]
pub struct EnvPloxy(Arc<EnvPloxyInner>);

pub type CallUnivSig = dyn (Fn(
        /*plugin id=*/ &str,
        /*symbol=*/ &str,
        /*arg=*/ &[u8],
        /*abi id=*/ u64,
        /*ploxy=*/ EnvPloxy,
    ) -> Result<Vec<u8>, BugiError>)
    + Send
    + Sync;

struct EnvPloxyInner {
    pub cache: Option<CachePloxy>,

    pub call_univ: Box<CallUnivSig>,
}

impl EnvPloxy {
    pub fn new(cacher: Option<&Cacher>, call_univ: Box<CallUnivSig>, plug_id: PluginId) -> Self {
        Self(Arc::new(EnvPloxyInner {
            cache: cacher.map(|cacher| CachePloxy {
                get_global: {
                    let cacher = cacher.clone();
                    Box::new(move |str| cacher.pop_global(str))
                },
                set_global: {
                    let cacher = cacher.clone();
                    Box::new(move |str, data| cacher.push_global(str, data))
                },
                get_cache: {
                    let cacher = cacher.clone();
                    Box::new(move || cacher.pop(plug_id))
                },
                set_cache: {
                    let cacher = cacher.clone();
                    Box::new(move |data| cacher.push(plug_id, data))
                },
            }),
            call_univ,
        }))
    }

    pub fn get_cache(&self) -> Option<CacheData> {
        self.0.cache.as_ref().map(|c| (c.get_cache)())?
    }

    pub fn set_cache(&self, data: CacheData) {
        if let Some(c) = self.0.cache.as_ref() {
            (c.set_cache)(data)
        }
    }

    pub fn call_univ_raw(
        &self,
        str: &str,
        symbol: &str,
        arg: &[u8],
        abi: u64,
    ) -> Result<Vec<u8>, BugiError> {
        (self.0.call_univ)(str, symbol, arg, abi, EnvPloxy(self.0.clone()))
    }

    pub fn call_univ<S: SerializeTag, Output: FromByte<S>>(
        &self,
        str: &str,
        symbol: &str,
        args: impl ParamListTo<S>,
    ) -> Result<Output, BugiError> {
        Ok(Output::from_byte(&self.call_univ_raw(
            str,
            symbol,
            &args.to_byte()?,
            S::get_abi_id(),
        )?)?)
    }

    pub fn get_global(&self, str: &str) -> Option<CacheData> {
        self.0.cache.as_ref().and_then(|c| (c.get_global)(str))
    }

    pub fn set_global(&self, str: &str, data: CacheData) {
        if let Some(c) = self.0.cache.as_ref() {
            (c.set_global)(str, data)
        }
    }
}

pub struct CachePloxy {
    pub get_global: Box<dyn (Fn(&str) -> Option<CacheData>) + Send + Sync>,
    pub set_global: Box<dyn (Fn(&str, CacheData)) + Send + Sync>,

    pub get_cache: Box<dyn (Fn() -> Option<CacheData>) + Send + Sync>,
    pub set_cache: Box<dyn (Fn(CacheData)) + Send + Sync>,
}
