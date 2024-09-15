use anyhow::{Context, Result};
use wasmtime::{Engine, Linker, Module, Store};

use crate::spec_type::PluginDesc;

pub(crate) type PluginID = u32;

pub struct PluginModule {
    pub(crate) id: PluginID,
    pub(crate) str_id: String,
    pub(crate) module: Module,
}

pub struct PluginRef {
    pub(crate) id: PluginID,
}

pub(crate) fn empty_import(engine: &Engine) -> Linker<()> {
    let mut res = Linker::new(engine);
    res.func_wrap(
        "bugi@v0",
        "call_univ_func",
        |_id_ptr: i32,
         _id_len: i32,
         _name_ptr: i32,
         _name_len: i32,
         _args_ptr: i32,
         _args_len: i32| { (0, 0) },
    )
    .unwrap();

    res
}

pub(crate) fn get_desc(engine: &Engine, module: &Module) -> Result<PluginDesc> {
    let mut store = Store::new(engine, ());
    let linker = empty_import(engine);

    let ins = linker.instantiate(&mut store, module)?;
    let desc = ins
        .get_typed_func::<(), (i32, i32)>(&mut store, "__bugi_v0_provide_desc")?
        .call(&mut store, ())?;

    let desc = PluginDesc::ser(
        (
            &ins.get_memory(&mut store, "memory")
                .with_context(|| "memery get error")?,
            &store,
        ),
        desc.0,
        desc.1,
    )?;

    Ok(desc)
}
