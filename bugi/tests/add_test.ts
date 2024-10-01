import { $ } from "jsr:@david/dax";

$.cd(import.meta);

$.cd("t-pdk");

const name = Deno.args[0];

const empty_script = `use bugi_pdk::{export, provide_desc};

provide_desc!("func");

#[export]
fn func() {
    _ = 1 + 1;
}
`

const cargo_add = `[[example]]
name = "t_${name}"
path = "src/t_${name}.rs"
crate-type = ["cdylib"]
`

await $`echo ${empty_script} > src/t_${name}.rs`;
await $`echo ${cargo_add} >> Cargo.toml`;