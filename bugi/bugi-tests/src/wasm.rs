use anyhow::Result;
use bugi::{RmpTag, Universe, WasmPlugin};

#[test]
fn wasm_call() -> Result<()> {
    let univ = Universe::new();
    let wasm = WasmPlugin::load(format!(
        "{}/wasm-plug.test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let pref = univ.add_plugin(wasm)?;

    let res = pref.call::<RmpTag, String>("reverse_string", ("ABCD".to_string(),))?;

    assert_eq!(res, "DCBA".to_string());

    Ok(())
}
