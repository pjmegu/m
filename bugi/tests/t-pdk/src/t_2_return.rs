use bugi_pdk::{export, provide_desc};

provide_desc!("func");

#[export]
fn func() -> i32 {
    1 + 1
}