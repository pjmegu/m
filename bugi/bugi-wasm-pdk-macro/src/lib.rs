use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    bugi_wasm_pdk_macro2::export_macro(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn plugin_id(name: TokenStream) -> TokenStream {
    bugi_wasm_pdk_macro2::plugin_id_macro(name.into()).into()
}
