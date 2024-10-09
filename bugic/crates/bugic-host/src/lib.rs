use std::collections::HashMap;

use bugic_core::{ParamListFrom, SerializeTag, ToByte, ERROR_ABI_ID};
use bugic_core::{
    PluginSystem,
    BugiError,
};

pub(crate) type HostPluginFuncRaw = Box<dyn (Fn(&[u8]) -> Vec<u8>) + Send + Sync>;

#[derive(Default)]
pub struct HostPlugin {
    funcs: HashMap<String, (u8, HostPluginFuncRaw)>,
}

impl HostPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host_func<
        SType: SerializeTag,
        Param: ParamListFrom<SType>,
        Result: ToByte<SType>,
    >(
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
        let func = self.funcs.get(symbol).ok_or(BugiError::PluginCallError("Symbol not found".to_string()))?;

        if abi != func.0 {
            return Err(BugiError::PluginAbiError(func.0));
        }
        
        Ok(func.1(param))
    }

    fn check_symbol_abi(
        &self,
        symbol: &str,
        abi: u8,
    ) -> Result<(), u8> {
        let (abi_arg_, _) = self.funcs.get(symbol).ok_or(ERROR_ABI_ID)?;
        if abi == *abi_arg_ {
            Ok(())
        } else {
            Err(*abi_arg_)
        }
    }
}

