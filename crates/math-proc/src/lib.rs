use quote::quote;

#[proc_macro]
pub fn hello_world(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote!{ "Hello, World!" }.into()
}
