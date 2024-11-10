use bugi_wasm_pdk::{export, plugin_id};

plugin_id!("wasm-test-plug");

#[export("reverse_string", RmpTag)]
fn reverse_string(str: String) -> String {
    str.chars().rev().collect()
}

#[export("one_zero", RmpTag)]
fn one_zero(_str: String) {
    // nothing to do
}

#[export("zero_one", RmpTag)]
fn zero_one() -> String {
    "TEST".to_string()
}
