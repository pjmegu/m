pub use bugic_share::*;

pub trait PluginSystem: Send + Sync {
    /// call a plugin function
    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi_arg: u8,
        abi_res: u8,
    ) -> Result<Vec<u8>, BugiError>;

    /// check the ABI of a symbol
    fn check_symbol_abi(
        &self,
        symbol: &str,
        abi_arg: u8,
        abi_res: u8,
    ) -> Result<(), (u8, u8)>;
}

#[derive(thiserror::Error, Debug)]
pub enum BugiError {
    #[error("cannot serialize: {0}")]
    CannotSerialize(#[from] bugic_share::SerializeError),

    #[error("the plugin ID already exists: {0}")]
    PluginIdExists(String),

    #[error("plugin is dropped")]
    PluginDropped,

    #[error("plugin call error: {0}")]
    PluginCallError(String),

    #[error("plugin abi error: {0}, {1}")]
    PluginAbiError(u8, u8),
}

/// Plugin Reference ID
pub type PluginId = u32;
