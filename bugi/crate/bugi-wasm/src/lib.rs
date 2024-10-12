use std::{collections::HashMap, sync::LazyLock};

static ENGINE: LazyLock<wasmtime::Engine> = LazyLock::new(|| {
    let mut config = wasmtime::Config::new();
    config.allocation_strategy(wasmtime::InstanceAllocationStrategy::pooling());
    wasmtime::Engine::new(&config).unwrap()
});

pub struct WasmPlugin {
    section: HashMap<String, Vec<u8>>,
    module: wasmtime::Module,
}

fn parse_custom_section(bin: &[u8]) -> HashMap<String, Vec<u8>>{
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
    fn raw_call(
        &self,
        symbol: &str,
        param: &[u8],
        abi: u8,
        ploxy: bugi_core::EnvPloxy,
    ) -> Result<Vec<u8>, bugi_core::BugiError> {
        const STORE_KEY: &str = "WasmPlugin-Store";
        
        let engine = ENGINE.clone();

        let mut store = match ploxy.get_global(STORE_KEY) {
            Some(store) => {
                store.downcast().unwrap()
            },
            None => {
                Box::new(wasmtime::Store::new(&engine, ()))
            }
        };

        let linker = wasmtime::Linker::new(&engine);
        let ins = linker.instantiate(&mut *store, &self.module).map_err(|err| {
            bugi_core::BugiError::PluginCallError(format!("Failed to wasm instantiate: {:?}", err))
        })?;

        
        ploxy.set_global(STORE_KEY, store);

        todo!()
    }
}