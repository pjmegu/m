use std::any::Any;

pub use bugi_share::*;

pub enum CacheType {
    CantCache,
    Cacheable,
    Cached(Box<dyn Any>),
}

type ResultType = Result<(Vec<u8>, Option<Box<dyn Any>>), BugiError>;
pub trait PluginSystem: Send + Sync {
    /// call a plugin function
    /// if cache is unit value, it means no cache
    fn raw_call(&self, symbol: &str, param: &[u8], abi: u8, cache: CacheType) -> ResultType;
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
