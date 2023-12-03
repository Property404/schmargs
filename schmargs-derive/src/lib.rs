mod derive_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
pub(crate) mod utils;

#[proc_macro_derive(Schmargs, attributes(arg))]
pub fn schmargs_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_impl::schmargs_derive_impl(input).unwrap()
}
