use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bugi_core::{BugiError, EnvPloxy, FromByte, ParamListFrom, ParamListTo, SerializeTag, ToByte};

use crate::plugin::PluginRef;

#[derive(Default, Clone)]
pub struct Overrider(Rc<RefCell<OverriderInner>>);

type OverrideFn = Box<dyn Fn(&[u8]) -> Result<Vec<u8>, BugiError> + 'static>;

#[derive(Default)]
pub struct OverriderInner {
    funcs: HashMap<(String, String), (u8, OverrideFn)>,
}

impl Overrider {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(OverriderInner {
            funcs: HashMap::new(),
        })))
    }

    pub fn add<SType: SerializeTag, Param: ParamListFrom<SType>, Result: ToByte<SType>>(
        &mut self,
        str_id: &str,
        symbol: &str,
        func: impl Fn(Param) -> Result + 'static,
    ) {
        self.0.borrow_mut().funcs.insert(
            (str_id.to_string(), symbol.to_string()),
            (
                SType::get_abi_id(),
                Box::new(move |arg| {
                    let arg = Param::from_byte(arg).map_err(BugiError::CannotSerialize)?;
                    let result = func(arg);
                    result.to_byte().map_err(BugiError::CannotSerialize)
                }),
            ),
        );
    }

    fn wrap_call_inner<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        pref: &PluginRef,
        sym: &str,
        param: impl ParamListTo<SType>,
        cacher: Option<&bugi_core::Cacher>,
    ) -> Result<Output, BugiError> {
        let univw = pref.univ_ref.clone();
        let s = self.clone();
        let id = pref.id;
        let ploxy = EnvPloxy::new(
            cacher,
            Box::new(move |str, symbol, arg, abi, ploxy| {
                if let Some(data) =
                    s.0.borrow()
                        .funcs
                        .get(&(str.to_string(), symbol.to_string()))
                {
                    if abi != data.0 {
                        return Err(BugiError::PluginAbiError(data.0));
                    }

                    return (data.1)(arg);
                }

                let univ = univw
                    .upgrade()
                    .ok_or_else(|| BugiError::PluginUniverseDropped)?;

                if str == "self" {
                    univ.call_raw_id(id, symbol, arg, abi, ploxy)
                } else {
                    univ.call_raw(str, symbol, arg, abi, ploxy)
                }
            }),
            pref.id,
        );

        pref.call_with_ploxy(sym, param, ploxy)
    }

    pub fn wrap_call<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        pref: &PluginRef,
        sym: &str,
        param: impl ParamListTo<SType>,
    ) -> Result<Output, BugiError> {
        self.wrap_call_inner::<SType, Output>(pref, sym, param, None)
    }

    pub fn wrap_call_cache<SType: SerializeTag, Output: FromByte<SType>>(
        &self,
        pref: &PluginRef,
        sym: &str,
        param: impl ParamListTo<SType>,
        cacher: &bugi_core::Cacher,
    ) -> Result<Output, BugiError> {
        self.wrap_call_inner(pref, sym, param, Some(cacher))
    }
}
