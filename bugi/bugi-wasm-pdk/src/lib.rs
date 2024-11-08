use std::alloc::Layout;

pub use bugi_wasm_pdk_macro::export;
pub use bugi_wasm_pdk_macro::plugin_id;

pub mod macro_prelude {
    pub use bugi_share::*;
}

#[no_mangle]
#[export_name = "bugi@v0_low_malloc"]
pub extern "C" fn alloc(len: u32) -> u32 {
    let layout = Layout::array::<u8>(len as usize).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        panic!("Can't Memory Alloc")
    } else {
        ptr as u32
    }
}

#[no_mangle]
#[export_name = "bugi@v0_low_free"]
pub extern "C" fn dealloc(ptr: u32, len: u32) {
    let layout = Layout::array::<u8>(len as usize).unwrap();
    unsafe { std::alloc::dealloc(ptr as *mut _, layout) }
}
