import { $ } from "jsr:@david/dax";
await $`cd ../simple-pdk && cargo build -r --example simple-pdk`;
await $`cp ../../../target/wasm32-unknown-unknown/release/examples/simple_pdk.wasm .`;
