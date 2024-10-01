use bugi_pdk::{export, provide_desc};

provide_desc!("t_1_empty");

#[export]
fn add() {
    _ = 1 + 1;
}
