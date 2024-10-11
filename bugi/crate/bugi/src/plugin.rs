use std::sync::Weak;

use bugi_core::{BugiError, PluginId, PluginSystem};
use bugi_share::{FromByte, ParamListTo, SerializeTag};

/// plugin (original)
pub struct Plugin {
    str_id: String,
    detail: Box<dyn PluginSystem>,
}

impl Plugin {
    /// Create a new Host Plugin
    pub fn new(str_id: &str, detail: impl PluginSystem + 'static) -> Self {
        Self {
            str_id: str_id.to_string(),
            detail: Box::new(detail),
        }
    }

    /// Get the string ID of the plugin
    pub fn get_str_id(&self) -> &str {
        &self.str_id
    }
}

/// Reference to a plugin
pub struct PluginRef {
    /// Weak reference to the plugin of this reference
    pref: Weak<Plugin>,

    /// plugin id
    id: PluginId,
}

impl PluginRef {
    /// make a new PluginRef
    pub(crate) fn new(pref: Weak<Plugin>, id: PluginId) -> Self {
        Self { pref, id }
    }

    /// Call the plugin
    pub fn call<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        symbol: &str,
        param: impl ParamListTo<SType>,
    ) -> Result<Output, BugiError> {
        let plug = self.pref.upgrade().ok_or(BugiError::PluginDropped)?;

        let param = param.to_byte().map_err(BugiError::CannotSerialize)?;
        let result = plug.detail.raw_call(symbol, &param, SType::get_abi_id(), None)?;
        Ok(Output::from_byte(&result.0)?)
    }

    /// Call with Cacher
    pub fn call_cache<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        symbol: &str,
        param: impl ParamListTo<SType>,
        cacher: &bugi_core::Cacher,
    ) -> Result<Output, BugiError> {
        let plug = self.pref.upgrade().ok_or(BugiError::PluginDropped)?;

        let param = param.to_byte().map_err(BugiError::CannotSerialize)?;
        let result = plug.detail.raw_call(symbol, &param, SType::get_abi_id(), Some(cacher.get_gcache(cacher.pop(self.id))))?;
        if let Some(cache) = result.1 {
            cacher.push(self.id, cache);
        }
        Ok(Output::from_byte(&result.0)?)
    }
}
