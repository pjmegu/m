use anyhow::Result;
use bugi::{HostPlugin, RmpTag, Universe, WasmPlugin};

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

#[test]
fn wasm_call_1_0() -> Result<()> {
    let univ = Universe::new();
    let wasm = WasmPlugin::load(format!(
        "{}/wasm-plug.test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let pref = univ.add_plugin(wasm)?;

    pref.call::<RmpTag, ()>("one_zero", ("ABCD".to_string(),))?;

    Ok(())
}

#[test]
fn wasm_call_0_1() -> Result<()> {
    let univ = Universe::new();
    let wasm = WasmPlugin::load(format!(
        "{}/wasm-plug.test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let pref = univ.add_plugin(wasm)?;

    let res = pref.call::<RmpTag, String>("zero_one", ())?;

    assert_eq!(res, "TEST".to_string());

    Ok(())
}

#[test]
fn call_univ_test() -> Result<()> {
    let univ = Universe::new();
    let mut host = HostPlugin::new("host");
    host.host_func::<RmpTag, (), _>("get_string", |_, _| "TEST".to_string());
    let _ = univ.add_plugin(host);
    let wasm = WasmPlugin::load(format!(
        "{}/wasm-plug.test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let pref = univ.add_plugin(wasm)?;

    let res = pref.call::<RmpTag, String>("call_univ_test", ())?;

    assert_eq!(res, "TEST".to_string());

    Ok(())
}
