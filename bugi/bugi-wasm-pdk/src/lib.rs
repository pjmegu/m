use std::alloc::Layout;

use bugi_share::FromByte;
use bugi_share::ParamListTo;
use bugi_share::SerializeTag;
use rmpv::Value;

pub use bugi_wasm_pdk_macro::export;
pub use bugi_wasm_pdk_macro::plugin_id;

pub mod macro_prelude {
    pub use bugi_share::*;
}

#[link(wasm_import_module = "bugi@v0")]
#[allow(improper_ctypes)]
extern "C" {
    fn call_univ(arg_ptr: u32, arg_len: u32) -> u64;
}

pub fn call<SType: SerializeTag, Output: FromByte<SType>>(
    id: &str,
    symbol: &str,
    param: impl ParamListTo<SType>,
) -> Output {
    let argv = Value::Map(vec![
        (Value::String("id".into()), Value::String(id.into())),
        (Value::String("name".into()), Value::String(symbol.into())),
        (
            Value::String("abi".into()),
            Value::Integer(SType::get_abi_id().into()),
        ),
        (
            Value::String("detail".into()),
            Value::Binary(param.to_byte().unwrap()),
        ),
    ]);

    let mut arg = Vec::new();
    rmpv::encode::write_value(&mut arg, &argv).unwrap();

    let arg_ptr = alloc(arg.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(arg.as_ptr(), arg_ptr as _, arg.len()) }

    let res = unsafe { call_univ(arg_ptr, arg.len() as u32) };

    let res_ptr = (res >> 32) as u32;
    let res_len = (res & 0xFFFFFFFF) as u32;

    let res: Vec<u8> =
        unsafe { std::slice::from_raw_parts(res_ptr as _, res_len as usize).to_vec() };

    dealloc(res_ptr, res_len);

    Output::from_byte(&res).unwrap()
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
