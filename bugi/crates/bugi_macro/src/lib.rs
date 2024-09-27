mod parse_export;

use std::collections::HashMap;

use parse_export::parse_export;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, Ident, ItemFn};

struct ExportAttrSetting {
    export_name: String,
    cacheable: bool,
    func_name: String,
}

type ParseResult = HashMap<String, DataType>;

enum DataType {
    String(String),
    Bool(bool),
}

pub fn macro_export_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let str = attr.to_string();
    let item = syn::parse2::<ItemFn>(item).unwrap();

    let (_, data) = parse_export(str.as_str()).unwrap();

    let setting = ExportAttrSetting {
        export_name: match data.get("export_name") {
            Some(d) => {
                if let DataType::String(s) = d {
                    s.clone()
                } else {
                    panic!("`export_name` value type is not string")
                }
            }
            None => item.sig.ident.to_string(),
        },
        cacheable: match data.get("cacheable") {
            Some(d) => {
                if let DataType::Bool(b) = d {
                    *b
                } else {
                    panic!("`cacheable` value type is not bool")
                }
            }
            None => false,
        },
        func_name: item.sig.ident.to_string(),
    };

    let name_call = format!("__bugi_v0_called_func_{}", &setting.export_name);
    let name_call_ident = format_ident!("__bugi_called_func_{}", &setting.export_name);
    let name_desc = format!("__bugi_v0_provide_func_desc_{}", setting.export_name);
    let name_desc_ident = format_ident!("__bugi_provide_func_desc_{}", setting.export_name);
    let cache = setting.cacheable;
    let func_name = Ident::new(&setting.func_name, Span::call_site());

    let mut arg_token = Vec::new();
    let mut arg_idents = Vec::new();
    for (index, arg) in item.clone().sig.inputs.iter().enumerate() {
        if let FnArg::Typed(arg) = arg {
            let ty = &arg.ty;
            let ident_v = format_ident!("args_{}", index);
            arg_token.push(quote! {
                let #ident_v = value_to_vec(args_arr.get(#index).unwrap());
                let #ident_v = from_slice::<#ty>(&#ident_v).unwrap();
            });
            arg_idents.push(ident_v);
        } else {
            panic!("`self` func is not supported")
        }
    }

    let gen_code = quote! {
        #item

        #[export_name = #name_desc]
        #[allow(improper_ctypes_definitions)]
        extern "C" fn #name_desc_ident() -> (u32,u32) {
            use ::bugi_pdk::macro_prelude::*;

            let mut bytes = Vec::new();
            write_value(
                &mut bytes,
                &Map(vec![(String(Utf8String::from("cacheable")), Boolean(#cache))]),
            )
            .expect("message pack encode error");
            copy_ptr(&bytes)
        }

        #[export_name = #name_call]
        #[allow(improper_ctypes_definitions)]
        extern "C" fn #name_call_ident(args_ptr: u32, args_len: u32) -> (u32, u32) {
            use ::bugi_pdk::macro_prelude::*;

            let mut args = slice_from_ptr_len(args_ptr, args_len);

            let args = read_value(&mut args).unwrap();

            if let Array(args_arr) = args {
                #[allow(clippy::get_first)]
                #(#arg_token)*

                let res = #func_name(#(#arg_idents),*);

                let res = to_vec_named(&res).unwrap();
                copy_ptr(&res)
            } else {
                panic!("internal error: `__bugi_v0_called_func_{}` args is not array", #name_call)
            }
        }
    };

    gen_code
}
