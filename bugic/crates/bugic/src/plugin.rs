use std::sync::Weak;

use crate::{
    host_plug::HostPlugin,
    param::{FromByte, ParamListTo, SerializeTag},
    BugiError, PluginSymbol,
};

pub struct Plugin {
    str_id: String,
    detail: Box<dyn PluginSystem>,
}

impl Plugin {
    /// Create a new Host Plugin
    pub fn make_host(str_id: String, host: HostPlugin) -> Self {
        Self {
            str_id,
            detail: Box::new(host),
        }
    }

    /// Get the string ID of the plugin
    pub fn get_str_id(&self) -> &String {
        &self.str_id
    }
}

pub trait PluginSystem: Send + Sync {
    fn raw_call(&self, symbol: &PluginSymbol, param: &[u8]) -> Result<Vec<u8>, BugiError>;
    fn check_symbol_abi(
        &self,
        symbol: &PluginSymbol,
        abi_arg: u8,
        abi_res: u8,
    ) -> Result<(), (u8, u8)>;
}

/// Reference to a plugin
pub struct PluginRef {
    // /// Plugin ID
    // id: PluginId,

    /// Weak reference to the plugin of this reference
    pref: Weak<Plugin>,

    // /// Weak reference to the universe of this plugin
    // uref: UniverseWeak,
}

impl PluginRef {
    /// make a new PluginRef
    pub(crate) fn new(pref: Weak<Plugin>) -> Self {
        Self { pref }
    }

    /// Call the plugin
    pub fn call<
        SInput: SerializeTag,
        SOutput: SerializeTag,
        Param: ParamListTo<SInput>,
        Output: FromByte<SOutput>,
    >(
        &self,
        symbol: String,
        param: Param,
    ) -> Result<Output, BugiError> {
        let plug = self.pref.upgrade().ok_or(BugiError::PluginDropped)?;

        plug.detail.check_symbol_abi(
            &symbol,
            SInput::get_abi_id(),
            SOutput::get_abi_id(),
        ).map_err(|(inp, out)| BugiError::PluginAbiError(inp, out))?;

        let param = param.to_byte().map_err(BugiError::CannotSerialize)?;
        let result = plug.detail.raw_call(&symbol, &param)?;
        Ok(Output::from_byte(&result)?)
    }
}
