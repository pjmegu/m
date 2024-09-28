use std::{
    borrow::BorrowMut,
    sync::{RwLock, Weak},
};

use anyhow::{Context, Result};
use wasmtime::{AsContextMut, Engine, Instance, Linker, Module, Store};

use crate::{
    cacher::PluginCacher,
    hostfunc::set_hostfunc,
    param::{PluginInput, PluginResult},
    spec_type::PluginDesc,
    universe::PluginUniverseWeak,
};

pub(crate) type PluginID = u32;

pub(crate) struct PluginModule {
    pub(crate) id: PluginID,
    // pub(crate) str_id: String,
    pub(crate) module: Module,
    // pub(crate) univ: PluginUniverseWeak,
}

/// Plug-in reference.
/// From here you can call the referenced plugin's function.
pub struct PluginRef {
    pub(crate) id: PluginID,
    pub(crate) univ: PluginUniverseWeak,
    pub(crate) module: Weak<RwLock<PluginModule>>,
}

pub(crate) struct PluginInstance {
    pub(crate) ins: Instance,
}

pub(crate) fn empty_import(engine: &Engine) -> Linker<()> {
    let mut res = Linker::new(engine);
    res.func_wrap(
        "bugi@v0",
        "call_univ_func",
        |_arg_ptr: u32, _arg_len: u32| (0, 0),
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

impl PluginRef {
    pub fn call<A: PluginInput, R: PluginResult>(
        &self,
        cacher: Option<PluginCacher>,
        name: String,
        args: &A,
    ) -> Result<R> {
        let binding = self
            .module
            .upgrade()
            .context("module get error")?;
        let module = binding
            .write()
            .unwrap();
        let engine = self.univ.upgrade().unwrap().get_engine();

        if let Some(cacher) = cacher {
            if let Some(ins) = cacher.get_ins(module.id) {
                let store = cacher.get_store();
                let mut store = store.write().unwrap();
                let result = ins.call::<A, R>((*store).borrow_mut(), &name, args)?;
                Ok(result)
            } else {
                let mut linker = Linker::new(&engine);
                set_hostfunc(&mut linker);
                let ins = linker
                    .instantiate(&mut *cacher.get_store().write().unwrap(), &module.module)?;
                cacher.add_ins(self.id, ins);
                self.call(Some(cacher), name, args)
            }
        } else {
            let cacher = PluginCacher::new(&self.univ.upgrade().unwrap());
            self.call(Some(cacher), name, args)
        }
    }
}

impl PluginInstance {
    pub(crate) fn call_low_mem_malloc(
        &self,
        store: &mut impl AsContextMut,
        len: u32,
    ) -> Result<u32> {
        let data = self
            .ins
            .get_typed_func::<u32, u32>(&mut *store, "__bugi_v0_low_mem_malloc")?
            .call(&mut *store, len)?;

        Ok(data)
    }

    pub(crate) fn call_low_mem_free(
        &self,
        store: &mut impl AsContextMut,
        ptr: u32,
        len: u32,
    ) -> Result<()> {
        self.ins
            .get_typed_func::<(u32, u32), ()>(&mut *store, "__bugi_v0_low_mem_free")?
            .call(&mut *store, (ptr, len))?;

        Ok(())
    }

    pub(crate) fn call<A: PluginInput, R: PluginResult>(
        &self,
        store: &mut impl AsContextMut,
        name: &String,
        args: &A,
    ) -> Result<R> {
        let arg_data = rmp_serde::to_vec_named(args)?;

        let mem_ptr = self.call_low_mem_malloc(&mut *store, arg_data.len() as u32)?;
        let mem = self
            .ins
            .get_memory(&mut *store, "memory")
            .context("memery get error")?;
        mem.write(&mut *store, mem_ptr as usize, &arg_data)?;

        let data = self
            .ins
            .get_typed_func::<(u32, u32), (u32, u32)>(
                &mut *store,
                format!("__bugi_v0_called_func_{}", name).as_str(),
            )?
            .call(&mut *store, (mem_ptr, arg_data.len() as u32))?;

        let mut buffer = vec![0; data.1 as usize];
        mem.read(&mut *store, data.0 as usize, &mut buffer)?;
        let data = rmp_serde::from_slice::<R>(&buffer)?;

        self.call_low_mem_free(&mut *store, mem_ptr, arg_data.len() as u32)?;

        Ok(data)
    }

    pub(crate) fn raw_call(
        &self,
        store: &mut impl AsContextMut,
        name: &String,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let mem_ptr = self.call_low_mem_malloc(&mut *store, args.len() as u32)?;
        let mem = self
            .ins
            .get_memory(&mut *store, "memory")
            .context("memery get error")?;
        mem.write(&mut *store, mem_ptr as usize, args)?;

        let data = self
            .ins
            .get_typed_func::<(u32, u32), (u32, u32)>(
                &mut *store,
                format!("__bugi_v0_called_func_{}", name).as_str(),
            )?
            .call(&mut *store, (mem_ptr, args.len() as u32))?;

        let mut buf = vec![0; data.1 as usize];
        mem.read(&mut *store, data.0 as usize, &mut buf)?;

        self.call_low_mem_free(&mut *store, mem_ptr, args.len() as u32)?;

        Ok(buf)
    }
}
