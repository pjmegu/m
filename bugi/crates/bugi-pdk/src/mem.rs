use std::alloc::Layout;

#[export_name = "__bugi_v0_low_mem_malloc"]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn mem_alloc(len: u32) -> u32 {
    let layout = Layout::array::<u8>(len as usize).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    ptr as u32
}

#[export_name = "__bugi_v0_low_mem_free"]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn mem_free(ptr: u32, len: u32) {
    let layout = Layout::array::<u8>(len as usize).unwrap();
    unsafe { std::alloc::dealloc(ptr as *mut u8, layout) };
}
