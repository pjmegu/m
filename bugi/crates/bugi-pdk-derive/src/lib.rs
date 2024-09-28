use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    bugi_macro::macro_export_attr(attr.into(), item.into()).into()
}
