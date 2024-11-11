use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, LitStr};

const WASM_SPEC_ID: &str = "bugi@v0_plugin_id";

pub fn plugin_id_macro(input: TokenStream) -> TokenStream {
    let name = parse2::<LitStr>(input).unwrap();
    let name = name.value().into_bytes();
    let len = name.len();
    quote! {
        #[link_section = #WASM_SPEC_ID]
        static __BUGI_PLUGIN_ID: [u8; #len] = [#(#name),*];
    }
}
