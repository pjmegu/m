use std::collections::HashMap;

use bugic_core::{ParamListFrom, SerializeTag, ToByte, ERROR_ABI_ID};
use bugic_core::{
    PluginSystem,
    BugiError,
};

pub(crate) type HostPluginFuncRaw = Box<dyn (Fn(&[u8]) -> Vec<u8>) + Send + Sync>;

#[derive(Default)]
pub struct HostPlugin {
    funcs: HashMap<String, (u8, u8, HostPluginFuncRaw)>,
}

impl HostPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host_func<
        SInput: SerializeTag,
        SOutput: SerializeTag,
        Param: ParamListFrom<SInput>,
        Result: ToByte<SOutput>,
        F: Fn(Param) -> Result + 'static + Send + Sync,
    >(
        &mut self,
        symbol: &str,
        func: F,
    ) {
        self.funcs.insert(
            symbol.to_string(),
            (
                SInput::get_abi_id(),
                SOutput::get_abi_id(),
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
    fn raw_call(&self, symbol: &str, param: &[u8], abi_arg: u8, abi_res: u8) -> Result<Vec<u8>, BugiError> {
        let func = self.funcs.get(symbol).ok_or(BugiError::PluginCallError("Symbol not found".to_string()))?;

        if abi_arg != func.0 || abi_res != func.1 {
            return Err(BugiError::PluginAbiError(func.0, func.1));
        }
        
        Ok(func.2(param))
    }

    fn check_symbol_abi(
        &self,
        symbol: &str,
        abi_arg: u8,
        abi_res: u8,
    ) -> Result<(), (u8, u8)> {
        let (abi_arg_, abi_res_, _) = self.funcs.get(symbol).ok_or((ERROR_ABI_ID, ERROR_ABI_ID))?;
        if abi_arg == *abi_arg_ && abi_res == *abi_res_ {
            Ok(())
        } else {
            Err((*abi_arg_, *abi_res_))
        }
    }
}

