use std::collections::HashMap;

use bugi_core::{BugiError, EnvPloxy, PluginSystem};
use bugi_core::{ParamListFrom, SerializeTag, ToByte};

pub(crate) type HostPluginFuncRaw =
    Box<dyn (Fn(&[u8], EnvPloxy) -> Result<Vec<u8>, BugiError>) + Send + Sync>;

#[derive(Default)]
pub struct HostPlugin {
    funcs: HashMap<String, (u8, HostPluginFuncRaw)>,
}

impl HostPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host_func<SType: SerializeTag, Param: ParamListFrom<SType>, Result: ToByte<SType>>(
        &mut self,
        symbol: &str,
        func: impl Fn(Param, EnvPloxy) -> Result + 'static + Send + Sync,
    ) {
        self.funcs.insert(
            symbol.to_string(),
            (
                SType::get_abi_id(),
                Box::new(move |arg, ploxy| {
                    let arg = Param::from_byte(arg).map_err(BugiError::CannotSerialize)?;
                    let result = func(arg, ploxy);
                    result.to_byte().map_err(BugiError::CannotSerialize)
                }),
            ),
        );
    }
}

impl PluginSystem for HostPlugin {
    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi: u8,
        ploxy: EnvPloxy,
    ) -> Result<Vec<u8>, BugiError> {
        let func = self
            .funcs
            .get(symbol)
            .ok_or(BugiError::PluginCallError(format!(
                "Symbol is not found: {}",
                symbol
            )))?;

        if abi != func.0 {
            return Err(BugiError::PluginAbiError(func.0));
        }

        match func.1(param, ploxy) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }
}
