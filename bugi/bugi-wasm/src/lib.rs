use core::panic;
use std::{collections::HashMap, sync::LazyLock};

use rmpv::ValueRef;
use wasmtime::Caller;

const SPEC_CALL_UNIV: (&str, &str) = ("bugi@v0", "call_univ");
const SPEC_PLUGIN_FUNC: &str = "bugi@v0_plugin_function_";
const SPEC_LOW_MALLOC: &str = "bugi@v0_low_malloc";
const SPEC_LOW_FREE: &str = "bugi@v0_low_free";
const SPEC_PLUG_ID: &str = "bugi@v0_plugin_id";

static ENGINE: LazyLock<wasmtime::Engine> = LazyLock::new(|| {
    let mut config = wasmtime::Config::new();
    config.allocation_strategy(wasmtime::InstanceAllocationStrategy::pooling());
    wasmtime::Engine::new(&config).unwrap()
});

pub struct WasmPlugin {
    section: HashMap<String, Vec<u8>>,
    module: wasmtime::Module,
}

fn parse_custom_section(bin: &[u8]) -> HashMap<String, Vec<u8>> {
    let parser = wasmparser::Parser::new(0);
    let parsed_data = parser.parse_all(bin);
    let mut res = HashMap::new();

    for payload in parsed_data {
        match payload {
            Ok(wasmparser::Payload::CustomSection(sec)) => {
                res.insert(sec.name().to_string(), sec.data().to_vec());
            }
            Ok(_) => {}
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    res
}

impl WasmPlugin {
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let engine = ENGINE.clone();
        let section = parse_custom_section(&std::fs::read(path.as_ref())?);
        let module = wasmtime::Module::from_file(&engine, path)?;
        Ok(Self { module, section })
    }

    pub fn load_bin(bin: &[u8]) -> anyhow::Result<Self> {
        let engine = ENGINE.clone();
        let section = parse_custom_section(bin);
        let module = wasmtime::Module::new(&engine, bin)?;
        Ok(Self { module, section })
    }
}

impl bugi_core::PluginSystem for WasmPlugin {
    fn str_id(&self) -> String {
        String::from_utf8(self.section.get(SPEC_PLUG_ID).unwrap().to_vec()).unwrap()
    }

    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi: u64,
        ploxy: bugi_core::EnvPloxy,
    ) -> Result<Vec<u8>, bugi_core::BugiError> {
        const STORE_KEY: &str = "WasmPlugin-Store";

        let engine = ENGINE.clone();

        let mut store = match ploxy.get_global(STORE_KEY) {
            Some(store) => store.downcast().unwrap(),
            None => Box::new(wasmtime::Store::new(&engine, ())),
        };

        let mut linker = wasmtime::Linker::new(&engine);
        let ploxy_c = ploxy.clone();
        linker
            .func_wrap(
                SPEC_CALL_UNIV.0,
                SPEC_CALL_UNIV.1,
                move |mut caller: Caller<'_, ()>, arg_ptr: u32, arg_len: u32| {
                    let malloc = caller
                        .get_export(SPEC_LOW_MALLOC)
                        .unwrap()
                        .into_func()
                        .unwrap()
                        .typed::<(u32,), u32>(&caller)
                        .unwrap();

                    let free = caller
                        .get_export(SPEC_LOW_FREE)
                        .unwrap()
                        .into_func()
                        .unwrap()
                        .typed::<(u32, u32), ()>(&caller)
                        .unwrap();

                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

                    let mut arg = vec![0; arg_len as usize];
                    if let Err(err) = memory.read(&caller, arg_ptr as usize, &mut arg) {
                        let err = format!("Can't Read Memory: \n{}", err);
                        panic!("<Bugi-Wasm> Found Error: {}", &err);
                    }

                    if let Err(err) = free.call(&mut caller, (arg_ptr, arg_len)) {
                        panic!("<Bugi-Wasm> Can't Dealloc Memory: {}", err);
                    }

                    let arg = rmpv::decode::read_value_ref(&mut arg.as_slice()).unwrap();

                    #[derive(Default)]
                    struct Arg {
                        id: String,
                        name: String,
                        abi: u64,
                        detail: Vec<u8>,
                    }

                    let arg = {
                        let mut a = Arg::default();
                        if let ValueRef::Map(vec) = arg {
                            for (name, value) in vec {
                                if let ValueRef::String(str) = name {
                                    fn get_string(v: ValueRef) -> String {
                                        if let ValueRef::String(str) = v {
                                            str.into_string().unwrap()
                                        } else {
                                            panic!(
                                                "<Bugi-Wasm> `call_univ`'s arg is not satisfiled."
                                            )
                                        }
                                    }
                                    fn get_u64(v: ValueRef) -> u64 {
                                        if let ValueRef::Integer(int) = v {
                                            int.as_u64().unwrap()
                                        } else {
                                            panic!(
                                                "<Bugi-Wasm> `call_univ`'s arg is not satisfiled."
                                            )
                                        }
                                    }
                                    fn get_bin(v: ValueRef) -> Vec<u8> {
                                        if let ValueRef::Binary(bin) = v {
                                            bin.to_vec()
                                        } else {
                                            panic!(
                                                "<Bugi-Wasm> `call_univ`'s arg is not satisfiled."
                                            )
                                        }
                                    }
                                    match str.as_str().unwrap() {
                                        "id" => {
                                            a.id = get_string(value);
                                        }

                                        "name" => {
                                            a.name = get_string(value);
                                        }

                                        "abi" => {
                                            a.abi = get_u64(value);
                                        }

                                        "detail" => {
                                            a.detail = get_bin(value);
                                        }

                                        _ => {}
                                    }
                                }
                            }
                        } else {
                            panic!("<Bugi-Wasm> `call_univ`'s arg is not map");
                        }

                        a
                    };

                    let result = ploxy_c.call_univ_raw(&arg.id, &arg.name, &arg.detail, arg.abi);

                    let res = match result {
                        Ok(v) => v,
                        Err(err) => panic!("<Bugi-Wasm> Call-Univ-Error: emit error during running function({}:{}) \n{}", &arg.id, &arg.name, err)
                    };

                    let mem = malloc.call(&mut caller, (res.len() as u32,));

                    let mem_ptr = match mem {
                        Ok(ptr) => ptr,
                        Err(err) => panic!("<Bugi-Wasm> Can't Alloc Memory: {}", err)
                    };

                    let result = memory.write(&mut caller, mem_ptr as usize, &res);
                    if let Err(err) = result {
                        panic!("<Bugi-Wasm> Can't Write Memory: {}", err)
                    }

                    (mem_ptr as u32, res.len() as u32)
                },
            )
            .unwrap();

        let ins = linker
            .instantiate(&mut *store, &self.module)
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(format!(
                    "Failed to wasm instantiate: {:?}",
                    err
                ))
            })?;

        let func = ins
            .get_typed_func::<(u32, u32, u64), u64>(
                &mut *store,
                &format!("{}{}", SPEC_PLUGIN_FUNC, symbol),
            )
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(format!(
                    "Symbol get error({}): {}",
                    symbol, err
                ))
            })?;

        let malloc = ins
            .get_typed_func::<(u32,), u32>(&mut *store, SPEC_LOW_MALLOC)
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(
                    format!("{SPEC_LOW_MALLOC} get error: {err}",),
                )
            })?;

        let free = ins
            .get_typed_func::<(u32, u32), ()>(&mut *store, SPEC_LOW_FREE)
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(format!("{SPEC_LOW_FREE} get error: {err}"))
            })?;

        let memory = ins.get_memory(&mut *store, "memory").ok_or_else(|| {
            bugi_core::BugiError::PluginCallError(
                "memory get error: `memory` is not exported".to_string(),
            )
        })?;

        let mem_ptr = malloc
            .call(&mut *store, (param.len() as u32,))
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(format!(
                    "can't alloc memory in `{SPEC_LOW_MALLOC}`: {err}"
                ))
            })?;

        if let Err(err) = memory.write(&mut *store, mem_ptr as usize, param) {
            return Err(bugi_core::BugiError::PluginCallError(format!(
                "can't write memory: {err}"
            )));
        }

        let res = func
            .call(&mut *store, (mem_ptr, param.len() as u32, abi))
            .map_err(|err| {
                bugi_core::BugiError::PluginCallError(format!(
                    "emit error during running `{symbol}`: {err}"
                ))
            })?;

        let res_ptr = (res >> 32) as u32;
        let res_len = (res & 0xFFFFFFFF) as u32;

        let mut res = vec![0; res_len as usize];
        if let Err(err) = memory.read(&mut *store, res_ptr as usize, &mut res) {
            return Err(bugi_core::BugiError::PluginCallError(format!(
                "can't read memory: {err}"
            )));
        }

        if let Err(err) = free.call(&mut *store, (res_ptr, res_len)) {
            return Err(bugi_core::BugiError::PluginCallError(format!(
                "can't dealloc memory: {err}"
            )));
        }

        ploxy.set_global(STORE_KEY, store);

        Ok(res)
    }
}
