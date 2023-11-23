#![allow(unused_imports)]
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    self, parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields,
    Lifetime, LifetimeParam, Token,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum SchmargsAttribute {
    ArgAttribute(ArgAttribute),
    DocAttribute(DocAttribute),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArgAttribute {
    short: Option<Option<char>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocAttribute {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttributeAggregate {
    doc: DocAttribute,
    arg: Option<ArgAttribute>,
}

fn parse_attribute(attr: &Attribute) -> SchmargsAttribute {
    match attr.meta {
        syn::Meta::List(ref list) => {
            assert!(attr.path().is_ident("arg"));

            if let Ok(path) = list.parse_args::<syn::Path>() {
                if path.is_ident("short") {
                    return SchmargsAttribute::ArgAttribute(ArgAttribute { short: Some(None) });
                } else {
                    panic!("Unknown attribute argument: {:?}", path.get_ident());
                }
            } else {
                let tok: syn::MetaNameValue = list.parse_args().unwrap();
                if tok.path.is_ident("short") {
                    let syn::Expr::Lit(lit) = tok.value else {
                        panic!("'short' argument expected literal");
                    };
                    let syn::Lit::Char(lit) = lit.lit else {
                        panic!("'short' argument expected character literal");
                    };
                    return SchmargsAttribute::ArgAttribute(ArgAttribute {
                        short: Some(Some(lit.value())),
                    });
                } else {
                    panic!("Unknown attribute argument: {:?}", tok.path.get_ident());
                }
            }
        }
        syn::Meta::NameValue(ref pair) => {
            assert!(attr.path().is_ident("doc"));
            let syn::Expr::Lit(ref value) = pair.value else {
                panic!("Expected literal attribute value ( i.e. doc comment)");
            };
            let syn::Lit::Str(ref value) = value.lit else {
                panic!("Expected str literal attribute value ( i.e. doc comment)");
            };
            return SchmargsAttribute::DocAttribute(DocAttribute {
                value: value.value().trim().into(),
            });
        }
        _ => panic!("Expected name-value pair attribute (i.e. doc comment)"),
    }
}

fn parse_attributes(attrs: &[Attribute]) -> AttributeAggregate {
    let mut doc = None;
    let mut arg = None;
    for attr in attrs {
        match parse_attribute(attr) {
            SchmargsAttribute::DocAttribute(attr) => {
                if doc.is_some() {
                    panic!("Was not expecting two doc attributes!");
                }
                doc = Some(attr);
            }
            SchmargsAttribute::ArgAttribute(attr) => {
                if arg.is_some() {
                    panic!("Was not expecting two arg attributes!");
                }
                arg = Some(attr);
            }
        }
    }

    AttributeAggregate {
        doc: doc.expect("Missing doc attribute"),
        arg,
    }
}

#[proc_macro_derive(Schmargs, attributes(arg))]
pub fn schmargs_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let description = parse_attributes(&input.attrs).doc.value;
    let default_lifetime = LifetimeParam::new(Lifetime::new("'a", Span::call_site().into()));
    let generics = input.generics.clone();
    let lifetime = generics.lifetimes().next().unwrap_or(&default_lifetime);

    let impl_generics = if input.generics.clone().lt_token.is_some() {
        quote! { #generics}
    } else {
        quote! { <#lifetime> }
    };

    let struct_generics = {
        let mut gen = quote! { < };
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
            let generic = &generic.ident;
            if first {
                first = false;
            } else {
                gen.extend(quote! { , });
            };
            gen.extend(quote! { #generic });
        }
        gen.extend(quote! { > });
        gen
    };

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.clone(),
        _ => panic!("expected a struct with named fields"),
    };

    let body = impl_fn_body(&fields);

    let gen = quote! {
        impl #impl_generics ::schmargs::Schmargs <#lifetime> for #name #struct_generics {
            fn description() -> &'static str {
                #description
            }
            fn parse(args: impl ::core::iter::Iterator<Item =  & #lifetime str>) -> ::core::result::Result<Self, ::schmargs::SchmargsError<#lifetime>> {
                #body
            }
        }
    };

    gen.into()
}

fn impl_fn_body(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let fields = &fields.named;
    let arg_flags: Vec<_> = fields
        .iter()
        .filter(|field| {
            parse_attributes(&field.attrs);
            field.ty.span().source_text().unwrap() == "bool"
        })
        .map(|field| (parse_attributes(&field.attrs), &field.ident))
        .collect();
    let arg_positionals: Vec<_> = fields
        .iter()
        .filter(|field| field.ty.span().source_text().unwrap() != "bool")
        .map(|field| &field.ident)
        .collect();

    let mut body = quote! {
        let args = ::schmargs::ArgumentIterator::from_args(args);
    };

    for (_, arg) in &arg_flags {
        body.extend(quote! {
            let mut #arg = false;
        });
    }

    for arg in &arg_positionals {
        body.extend(quote! {
            let mut #arg = ::core::option::Option::None;
        });
    }

    let match_body = {
        let mut gen: proc_macro2::TokenStream = Default::default();
        for (attr, arg) in &arg_flags {
            gen.extend(quote! {
                ::schmargs::Argument::LongFlag(stringify!(#arg)) => {
                    #arg = true;
                },
            });
            if let Some(ArgAttribute {
                short: Some(Some(short)),
            }) = attr.arg
            {
                gen.extend(quote! {
                    ::schmargs::Argument::ShortFlag(#short) => {
                        #arg = true;
                    },
                });
            }
        }

        let (num, positional): (Vec<usize>, Vec<&Option<proc_macro2::Ident>>) =
            arg_positionals.iter().enumerate().unzip();
        let (num, positional) = (num.into_iter(), positional.into_iter());
        gen.extend(quote! {
            ::schmargs::Argument::Positional(value) => {
                match pos_count {
                #(
                    #num => {#positional = Some(::schmargs::SchmargsField::parse_str(value)?);},
                )*
                    _ => {return ::core::result::Result::Err(::schmargs::SchmargsError::TooManyArguments);}
                }
                pos_count += 1;
            },
            arg=> {::core::result::Result::Err(::schmargs::SchmargsError::NoSuchOption(arg))?;}
        });

        gen
    };

    let return_body = {
        let mut body: proc_macro2::TokenStream = Default::default();

        for (_, arg) in arg_flags {
            body.extend(quote! {
                #arg,
            });
        }

        for arg in arg_positionals {
            body.extend(quote! {
                #arg: #arg.ok_or(
                    ::schmargs::SchmargsError::NotEnoughArguments
                )?,
            });
        }

        body
    };

    body.extend(quote! {

        let mut pos_count = 0;

        for arg in args {
            match arg {
                #match_body
            }
        }

        Ok(Self {
            #return_body
        })
    });

    body
}
