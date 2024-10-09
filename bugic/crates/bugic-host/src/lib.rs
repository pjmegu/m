use std::collections::HashMap;

use bugic_core::{BugiError, PluginSystem};
use bugic_core::{ParamListFrom, SerializeTag, ToByte};

pub(crate) type HostPluginFuncRaw = Box<dyn (Fn(&[u8]) -> Vec<u8>) + Send + Sync>;

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
        func: impl Fn(Param) -> Result + 'static + Send + Sync,
    ) {
        self.funcs.insert(
            symbol.to_string(),
            (
                SType::get_abi_id(),
                Box::new(move |arg| {
                    let arg = Param::from_byte(arg).unwrap();
                    let result = func(arg);
                    result.to_byte().unwrap()
                }),
            ),
        );
    }
}

impl PluginSystem for HostPlugin {
    fn raw_call(&self, symbol: &str, param: &[u8], abi: u8) -> Result<Vec<u8>, BugiError> {
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

        Ok(func.1(param))
    }
}
