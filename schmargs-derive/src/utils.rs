#![allow(unused_imports)]
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    self, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields, Generics, Lifetime,
    LifetimeParam,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CopyGenericsBoundOption {
    WithoutBounds,
    WithBounds,
}

// Copy inner generics, with or without trait bounds
pub(crate) fn copy_generics(
    generics: &Generics,
    bounds: CopyGenericsBoundOption,
) -> proc_macro2::TokenStream {
    let mut gen = quote! {};
    let mut first = true;
    for generic in generics.lifetimes() {
        if first {
            first = false;
        } else {
            gen.extend(quote! { , });
        };
        gen.extend(quote! { #generic });
    }
    for generic in generics.type_params() {
        if first {
            first = false;
        } else {
            gen.extend(quote! { , });
        };
        if bounds == CopyGenericsBoundOption::WithBounds {
            gen.extend(quote! { #generic });
        } else {
            let generic = &generic.ident;
            gen.extend(quote! { #generic });
        }
    }
    gen
}
