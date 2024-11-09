#![no_main]

use bugi_wasm_pdk::{export, plugin_id};

plugin_id!("wasm-test-plug");

#[export("reverse_string", RmpTag)]
fn reverse_string(str: String) -> String {
    str.chars().rev().collect()
}
