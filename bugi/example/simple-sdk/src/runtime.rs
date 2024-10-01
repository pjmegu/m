use bugi_sdk::PluginUniverse;
use serde::{Deserialize, Serialize};

const PLUGIN_BIN: &[u8] = include_bytes!("../simple_pdk.wasm");

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Test {
    s: String,
}

fn main() {
    let univ = PluginUniverse::new();
    let pref = univ.load_plugin_from_binary(PLUGIN_BIN).unwrap();
    let cacher = univ.make_cacher();
    pref.call::<_, (Test, Test)>(
        Some(cacher.clone()),
        "ttttt".to_string(),
        &(Test {
            s: "hello".to_string(),
        },),
    )
    .unwrap();
    // dbg!(result);
}
