mod derive_impl;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Schmargs, attributes(arg, schmargs))]
pub fn schmargs_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_impl::schmargs_derive_impl(input).unwrap()
}
