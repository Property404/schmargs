use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use quote::quote;
use syn::Generics;

pub(crate) trait TokenTreeExt {
    fn unwrap_as_literal(self) -> Literal;
    fn unwrap_as_ident(self) -> Ident;
}

impl TokenTreeExt for TokenTree {
    fn unwrap_as_literal(self) -> Literal {
        match self {
            Self::Literal(val) => val,
            _ => {
                panic!("Expected literal, but unwrapped something else");
            }
        }
    }

    fn unwrap_as_ident(self) -> Ident {
        match self {
            Self::Ident(val) => val,
            _ => {
                panic!("Expected ident, but unwrapped something else");
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CopyGenericsBoundOption {
    WithoutBounds,
    WithBounds,
}

// Copy inner generics, with or without trait bounds
pub(crate) fn copy_generics(generics: &Generics, bounds: CopyGenericsBoundOption) -> TokenStream {
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
