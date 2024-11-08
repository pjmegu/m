use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse2, FnArg, ItemFn, LitStr, ReturnType};

const WASM_SPEC_FUNC: &str = "bugi@v0_plugin_function_";
pub fn export_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (name, abi_type) = {
        let name;

        let mut iter = attr.into_iter().fuse();
        match iter.next() {
            Some(arg) => {
                name = parse2::<LitStr>(arg.into()).unwrap();
            }
            None => {
                panic!("not found first arg")
            }
        }

        let _ = iter.next();

        let ident = iter.collect::<TokenStream>();
        (name, ident)
    };

    let fn_item = parse2::<ItemFn>(item.clone()).unwrap();
    let arg_types = fn_item
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(ptype) => Some(&ptype.ty),
            _ => None,
        })
        .collect::<Vec<_>>();

    let fn_name = fn_item.sig.ident;
    let arg_get_token = if arg_types.is_empty() {
        quote! {
            let res = #fn_name();
        }
    } else {
        let i = (0..arg_types.len()).map(syn::Index::from);
        quote! {
            let arg: &[u8] = unsafe { std::slice::from_raw_parts(arg_ptr as *const _, arg_len as usize) };
            type ArgTuple = (#(#arg_types),*,);
            let arg: ArgTuple = <ArgTuple as FromByte<#abi_type>>::from_byte(arg).unwrap();
            let res = #fn_name(#(arg.#i),*);
        }
    };

    let return_token = match fn_item.sig.output {
        ReturnType::Default => {
            quote! {
                (0, 0)
            }
        }

        ReturnType::Type(_, ref ty) => {
            let return_type = ty.to_token_stream();
            quote! {

            type ReturnType = #return_type;
            let res = <ReturnType as ToByte<#abi_type>>::to_byte(&res).unwrap();
            let ptr = ::bugi_wasm_pdk::alloc(res.len() as u32);
            unsafe {
                std::ptr::copy_nonoverlapping(res.as_ptr(), ptr as *mut _, res.len());
            }

            (ptr, res.len() as u32)
            }
        }
    };

    let fn_name_ident = format_ident!("__bugi_func_{}", &name.value());
    let fn_name_export = format!("{}{}", WASM_SPEC_FUNC, &name.value());
    let token = quote! {
    #[no_mangle]
    #[export_name = #fn_name_export]
    extern "C" fn #fn_name_ident(arg_ptr: u32, arg_len: u32, abi_type: u64) -> (u32, u32) {
        use ::bugi_wasm_pdk::macro_prelude::*;
        if <#abi_type as SerializeTag>::get_abi_id() != abi_type {
                panic!("ABI Type(id: {}) is not match this function(id: {})", abi_type, <#abi_type as SerializeTag>::get_abi_id());
            }
        #arg_get_token
        #return_token
    }

            #item
        };
    token
}
