use bugi_pdk::{export, provide_desc};

provide_desc!("func");

#[export]
fn func(i: i32, j: i32) -> i32 {
    i + j
}
