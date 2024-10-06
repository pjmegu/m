use std::collections::HashMap;

use crate::{
    param::{ParamListFrom, SerializeTag, ToByte},
    plugin::PluginSystem,
    BugiError, PluginSymbol,
};

pub(crate) type HostPluginFuncRaw = Box<dyn (Fn(&[u8]) -> Vec<u8>) + Send + Sync>;

#[derive(Default)]
pub struct HostPlugin {
    funcs: HashMap<PluginSymbol, (u8, u8, HostPluginFuncRaw)>,
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
        symbol: String,
        func: F,
    ) {
        self.funcs.insert(
            symbol,
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
    fn raw_call(&self, symbol: &PluginSymbol, param: &[u8]) -> Result<Vec<u8>, BugiError> {
        Ok(self.funcs.get(symbol).ok_or::<BugiError>(
            BugiError::PluginCallError("Symbol not found".to_string()),
        )?.2(param))
    }

    fn check_symbol_abi(
        &self,
        symbol: &PluginSymbol,
        abi_arg: u8,
        abi_res: u8,
    ) -> Result<(), (u8, u8)> {
        let (abi_arg_, abi_res_, _) = self.funcs.get(symbol).ok_or((0xFF, 0xFF))?;
        if abi_arg == *abi_arg_ && abi_res == *abi_res_ {
            Ok(())
        } else {
            Err((*abi_arg_, *abi_res_))
        }
    }
}
