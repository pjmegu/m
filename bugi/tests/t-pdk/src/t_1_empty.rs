use bugi_pdk::{export, provide_desc};

provide_desc!("func");

#[export]
fn func() {
    _ = 1 + 1;
}
