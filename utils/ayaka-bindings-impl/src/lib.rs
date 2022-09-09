use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Ident, ItemFn};

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = input.clone();
    let func = parse_macro_input!(func as ItemFn);
    let name = func.sig.ident;
    let name_str = name.to_string();
    let expname = parse_str::<Ident>(&format!("__{}", name_str)).unwrap();
    let input = proc_macro2::TokenStream::from(input);
    let export_func = quote! {
        #[doc(hidden)]
        #[allow(unsafe_code)]
        #[export_name = #name_str]
        unsafe extern "C" fn #expname(len: usize, data: *const u8) -> u64 {
            ::ayaka_bindings::__export(len, data, #name)
        }
        #input
    };
    TokenStream::from(export_func)
}
