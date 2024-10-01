use std::sync::OnceLock;

use wasmtime::{Caller, Linker, Val};

use crate::cacher::PluginCacher;

pub(crate) fn set_hostfunc(linker: &mut Linker<OnceLock<PluginCacher>>) {
    linker
        .func_wrap(
            "bugi@v0",
            "call_univ_func",
            |mut caller: Caller<'_, OnceLock<PluginCacher>>, arg_ptr: u32, arg_len: u32| {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut args = vec![0; arg_len as usize];
                memory.read(&caller, arg_ptr as usize, &mut args).unwrap();

                // call memory free
                let f_free = caller
                    .get_export("__bugi_v0_low_mem_free")
                    .unwrap()
                    .into_func()
                    .unwrap();
                let mut results = Vec::new();
                f_free
                    .call(
                        &mut caller,
                        &[Val::I32(arg_ptr as i32), Val::I32(arg_len as i32)],
                        &mut results,
                    )
                    .unwrap();

                // decode args
                let args = rmpv::decode::read_value(&mut &args[..]).unwrap();

                // analyze args
                struct ArgData {
                    id: String,
                    name: String,
                    args: Vec<u8>,
                }

                let mut arg_data = ArgData {
                    id: String::new(),
                    name: String::new(),
                    args: Vec::new(),
                };

                for (key, val) in args.as_map().unwrap() {
                    if key.as_str().unwrap() == "id" {
                        arg_data.id = val.as_str().unwrap().to_string();
                    } else if key.as_str().unwrap() == "name" {
                        arg_data.name = val.as_str().unwrap().to_string();
                    } else if key.as_str().unwrap() == "args" {
                        let mut buf = Vec::new();
                        rmpv::encode::write_value(&mut buf, val).unwrap();
                        arg_data.args = buf;
                    } else {
                        panic!("unknown key: {}", key.as_str().unwrap());
                    }
                }

                // instantiate phase
                let cacher = caller.data().get().unwrap();
                let univ = cacher.get_univ();

                let plug_id = univ.get_str_id(&arg_data.id).unwrap();

                let ins = cacher.get_or_add_ins(plug_id, || {
                    let binding = univ.get_module(plug_id).unwrap();
                    let module = &binding.read().unwrap().module;
                    let mut linker = Linker::new(&univ.get_engine());
                    set_hostfunc(&mut linker);

                    let ins = linker
                        .instantiate(&mut *cacher.get_store().write().unwrap(), module)
                        .unwrap();

                    ins
                });

                // call phase
                let f_called = ins
                    .raw_call(
                        &mut *cacher.get_store().write().unwrap(),
                        &arg_data.name,
                        &arg_data.args,
                    )
                    .unwrap();

                // malloc
                let f_malloc = caller
                    .get_export("__bugi_v0_low_mem_malloc")
                    .unwrap()
                    .into_func()
                    .unwrap();
                let mut ptr = Vec::new();
                f_malloc
                    .call(&mut caller, &[Val::I32(f_called.len() as i32)], &mut ptr)
                    .unwrap();

                // write result
                memory
                    .write(&mut caller, ptr[0].unwrap_i32() as usize, &f_called)
                    .unwrap();

                // return ptr adn len
                (ptr[0].unwrap_i32() as u32, f_called.len() as u32)
            },
        )
        .unwrap();
}
