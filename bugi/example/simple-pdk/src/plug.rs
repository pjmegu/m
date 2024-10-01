use bugi_pdk::{export, provide_desc};
use serde::{Deserialize, Serialize};

provide_desc!("test");

#[derive(Serialize, Deserialize, Clone)]
struct Test {
    s: String,
}

#[export(export_name = "ttttt")]
fn test(t: Test) -> (Test, Test) {
    (t.clone(), t)
}
