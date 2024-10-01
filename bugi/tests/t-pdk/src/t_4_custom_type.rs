use bugi_pdk::{export, provide_desc};
use serde::{Deserialize, Serialize};

provide_desc!("func");

#[derive(Deserialize)]
struct Args {
    i: i32,
    j: i32,
}

#[derive(Deserialize)]
struct Args2 {
    k: i32,
    l: i32,
}

#[derive(Serialize)]
struct Res {
    res: f32,
}

#[export]
fn func(a: Args, b: Args2) -> Res {
    Res {
        res: ((a.i + b.k) as f32).sin(),
    }
}
