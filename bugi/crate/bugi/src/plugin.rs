use std::sync::Weak;

use bugi_core::{BugiError, EnvPloxy, PluginId, PluginSystem};
use bugi_share::{FromByte, ParamListTo, SerializeTag};

use crate::UniverseWeak;

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

    pub(crate) fn call_raw(
        &self,
        symbol: &str,
        arg: &[u8],
        abi: u8,
        ploxy: EnvPloxy,
    ) -> Result<Vec<u8>, BugiError> {
        self.detail.raw_call(symbol, arg, abi, ploxy)
    }
}

/// Reference to a plugin
pub struct PluginRef {
    /// Weak reference to the plugin of this reference
    pub(crate) pref: Weak<Plugin>,

    pub(crate) univ_ref: UniverseWeak,

    /// plugin id
    pub(crate) id: PluginId,
}

impl PluginRef {
    /// make a new PluginRef
    pub(crate) fn new(pref: Weak<Plugin>, id: PluginId, univ_ref: UniverseWeak) -> Self {
        Self { pref, id, univ_ref }
    }

    /// Call the plugin
    pub fn call<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        symbol: &str,
        param: impl ParamListTo<SType>,
    ) -> Result<Output, BugiError> {
        let univw = self.univ_ref.clone();
        let id = self.id;
        let env_plox = EnvPloxy::new(
            None,
            Box::new(move |str, symbol, arg, abi, ploxy| {
                let univ = univw
                    .upgrade()
                    .ok_or_else(|| BugiError::PluginUniverseDropped)?;
                if str == "self" {
                    univ.call_raw_id(id, symbol, arg, abi, ploxy)
                } else {
                    univ.call_raw(str, symbol, arg, abi, ploxy)
                }
            }),
            self.id,
        );
        self.call_with_ploxy(symbol, param, env_plox)
    }

    /// Call with Cacher
    pub fn call_cache<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        symbol: &str,
        param: impl ParamListTo<SType>,
        cacher: &bugi_core::Cacher,
    ) -> Result<Output, BugiError> {
        let univw = self.univ_ref.clone();
        let id = self.id;
        let env_plox = EnvPloxy::new(
            Some(cacher),
            Box::new(move |str, symbol, arg, abi, ploxy| {
                let univ = univw
                    .upgrade()
                    .ok_or_else(|| BugiError::PluginUniverseDropped)?;
                if str == "self" {
                    univ.call_raw_id(id, symbol, arg, abi, ploxy)
                } else {
                    univ.call_raw(str, symbol, arg, abi, ploxy)
                }
            }),
            self.id,
        );
        self.call_with_ploxy(symbol, param, env_plox)
    }

    pub(crate) fn call_with_ploxy<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        symbol: &str,
        param: impl ParamListTo<SType>,
        ploxy: EnvPloxy,
    ) -> Result<Output, BugiError> {
        let plug = self.pref.upgrade().ok_or(BugiError::PluginDropped)?;

        let param = param.to_byte().map_err(BugiError::CannotSerialize)?;

        let result = plug.call_raw(symbol, &param, SType::get_abi_id(), ploxy)?;

        Ok(Output::from_byte(&result)?)
    }
}
