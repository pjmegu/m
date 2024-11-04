use core::panic;
use std::{collections::HashMap, sync::LazyLock};

use rmpv::ValueRef;
use wasmtime::Caller;

static ENGINE: LazyLock<wasmtime::Engine> = LazyLock::new(|| {
    let mut config = wasmtime::Config::new();
    config.allocation_strategy(wasmtime::InstanceAllocationStrategy::pooling());
    wasmtime::Engine::new(&config).unwrap()
});

pub struct WasmPlugin {
    name: String,
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
    pub fn load(name: &str, path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let engine = ENGINE.clone();
        let section = parse_custom_section(&std::fs::read(path.as_ref())?);
        let module = wasmtime::Module::from_file(&engine, path)?;
        Ok(Self {
            module,
            section,
            name: name.to_string(),
        })
    }

    pub fn load_bin(name: &str, bin: &[u8]) -> anyhow::Result<Self> {
        let engine = ENGINE.clone();
        let section = parse_custom_section(bin);
        let module = wasmtime::Module::new(&engine, bin)?;
        Ok(Self {
            module,
            section,
            name: name.to_string(),
        })
    }
}

impl bugi_core::PluginSystem for WasmPlugin {
    fn str_id(&self) -> String {
        self.name.clone()
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
                "bugi@v0",
                "call_univ",
                move |mut caller: Caller<'_, ()>, arg_ptr: u32, arg_len: u32| {
                    let malloc = caller
                        .get_export("bugi@v0_low_malloc")
                        .unwrap()
                        .into_func()
                        .unwrap()
                        .typed::<(u32,), (u32, u32)>(&caller)
                        .unwrap();

                    let free = caller
                        .get_export("bugi@v0_low_free")
                        .unwrap()
                        .into_func()
                        .unwrap()
                        .typed::<(u32, u32), u32>(&caller)
                        .unwrap();

                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

                    let mut arg = vec![0; arg_len as usize];
                    let result = memory.read(&caller, arg_ptr as usize, &mut arg);
                    if let Err(err) = result {
                        let err = format!("Can't Read Memory: \n{}", err);
                        println!("<Bugi-Wasm> Found Error: {}", &err);
                        return (1, 0, 0);
                    }

                    let result = free.call(&mut caller, (arg_ptr, arg_len));

                    if let Err(err) = result {
                        panic!("<Bugi-Wasm> Can't Dealloc Memory: {}", err);
                    } else if result.unwrap() != 0 {
                        panic!("<Bugi-Wasm> Can't Dealloc Memory (Plugin side Error)")
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

                    let res = if let Err(err) = result {
                        panic!("<Bugi-Wasm> Call-Univ-Error: emit error during running function({}:{}) \n{}", &arg.id, &arg.name, err);
                    } else {
                            result.unwrap()
                    };





                    (0, 0, 0)
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

        ploxy.set_global(STORE_KEY, store);

        todo!()
    }
}
