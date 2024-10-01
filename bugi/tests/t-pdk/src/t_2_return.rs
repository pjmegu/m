use bugi_pdk::{export, provide_desc};

provide_desc!("t_2_return");

#[export]
fn return_v() -> i32 {
    1 + 1
}
