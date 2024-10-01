import { $ } from "jsr:@david/dax";

const pwd = Deno.cwd();
$.cd(import.meta);

// build wasms
const wasm_lists = [
    "t_1_empty",
    "t_2_return",
    "t_3_get_args",
    "t_4_custom_type",
];

for (const wasm of wasm_lists) {
    await $`cd t-pdk && cargo build -r --example ${wasm}`;
}

// copy wasms to sdk test folder
for (const wasm of wasm_lists) {
    await $`cp ./../../target/wasm32-unknown-unknown/release/examples/${wasm}.wasm ./t-sdk/wasms/.`;
}

$.cd(pwd);
