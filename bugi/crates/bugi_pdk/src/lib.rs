pub mod mem;

#[macro_export]
macro_rules! provide_desc {
    ($str_id:literal) => {
        #[export_name = "__bugi_v0_provide_desc"]
        #[allow(improper_ctypes_definitions)]
        extern "C" fn __provide_desc() -> (u32, u32) {
            use ::bugi_pdk::macro_prelude::*;

            let mut bytes = Vec::new();
            write_value(
                &mut bytes,
                &Map(vec![(
                    String(Utf8String::from("string_id")),
                    String(Utf8String::from($str_id)),
                )]),
            )
            .expect("message pack encode Error");
            copy_ptr(&bytes)
        }
    };
}

pub use bugi_pdk_derive::export;

pub mod macro_prelude {
    use std::slice;

    pub use ::rmp_serde::from_slice;
    pub use ::rmp_serde::to_vec_named;
    pub use ::rmpv::decode::read_value;
    pub use ::rmpv::encode::write_value;
    pub use ::rmpv::Utf8String;
    pub use ::rmpv::Value;
    pub use ::rmpv::Value::*;
    pub use ::std::mem::forget;
    pub use ::std::vec::Vec;

    pub fn slice_from_ptr_len<'a>(ptr: u32, len: u32) -> &'a [u8] {
        unsafe { slice::from_raw_parts(ptr as *mut _, len as usize) }
    }

    pub fn value_to_vec(v: &Value) -> Vec<u8> {
        let mut vec = Vec::new();
        rmpv::encode::write_value(&mut vec, v).unwrap();
        vec
    }

    pub fn copy_ptr(v: &[u8]) -> (u32, u32) {
        let layout = std::alloc::Layout::array::<u8>(v.len()).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe {
            std::ptr::copy_nonoverlapping(v.as_ptr(), ptr, v.len());
        };

        (ptr as u32, v.len() as u32)
    }
}
