use crate::utils::TokenTreeExt;
use anyhow::{bail, Result};
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::{
    self, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields, Lifetime,
    LifetimeParam,
};

#[derive(Debug, Clone)]
enum SchmargsAttribute {
    Arg(ArgAttribute),
    Doc(DocAttribute),
    TopLevel(TopLevelAttribute),
}

#[derive(Debug, Clone)]
struct TopLevelAttribute {
    // What kind of string this should iterate over
    // e.g. String or &str (default: &str)
    iterates_over: Option<Ident>,
    // Name of the program
    name: Option<Literal>,
}

#[derive(Debug, Clone)]
struct ArgAttribute {
    short: Option<Option<Literal>>,
    long: Option<Option<Literal>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocAttribute {
    value: String,
}

#[derive(Debug, Clone)]
struct AttributeAggregate {
    doc: DocAttribute,
    arg: Option<ArgAttribute>,
    top_level: Option<TopLevelAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgKind {
    Flag,
    Option,
    Positional,
}

#[derive(Debug, Clone)]
struct Arg {
    attr: AttributeAggregate,
    ident: Ident,
    is_bool: bool,
    is_option: bool,
}

impl Arg {
    fn kind(&self) -> ArgKind {
        if self.attr.arg.is_some() {
            if self.is_bool {
                ArgKind::Flag
            } else {
                ArgKind::Option
            }
        } else {
            ArgKind::Positional
        }
    }

    fn short(&self) -> Option<Literal> {
        if let Some(ArgAttribute {
            short: Some(short), ..
        }) = &self.attr.arg
        {
            return Some(short.clone().unwrap_or_else(|| {
                Literal::character(self.ident.to_string().chars().next().unwrap())
            }));
        }
        None
    }

    // Return as "__schmargs_ident_<ident>"
    fn unique_ident(&self) -> Ident {
        let ident = String::from("__schmargs_ident_") + &self.ident.to_string();
        Ident::new(&ident, self.ident.span())
    }

    // Return as "--long"
    fn long(&self) -> Option<String> {
        if let Some(ArgAttribute {
            long: Some(long), ..
        }) = &self.attr.arg
        {
            let long: String = long
                .clone()
                .map(|v| v.to_string())
                .unwrap_or_else(|| self.ident.to_string());
            let long = String::from("--")
                + &snailquote::unescape(&long).expect("Failed to unescape string");
            return Some(long);
        }
        None
    }
}

fn parse_attribute(attr: &Attribute) -> Result<SchmargsAttribute> {
    match attr.meta {
        syn::Meta::List(ref list) => {
            let tokens = list.parse_args::<TokenStream>().unwrap();

            let mut map = HashMap::new();

            let mut key = None;
            for token in tokens {
                match token {
                    ident @ TokenTree::Ident(_) => {
                        if let Some(key) = key.take() {
                            map.insert(key, Some(ident));
                        } else {
                            key = Some(ident.to_string());
                        }
                    }
                    TokenTree::Punct(punct) => match punct.as_char() {
                        ',' => {
                            if let Some(key) = key.take() {
                                map.insert(key, None);
                            }
                        }
                        '=' => {}
                        _ => {
                            bail!("Unexpected punctuation in attribute");
                        }
                    },
                    literal @ TokenTree::Literal(_) => {
                        if let Some(key) = key.take() {
                            map.insert(key, Some(literal));
                        } else {
                            bail!("Unexpected literal in attribute")
                        }
                    }
                    TokenTree::Group(_) => bail!("Unexpected token tree type"),
                }
            }
            if let Some(key) = key.take() {
                map.insert(key, None);
            }

            let return_value = if attr.path().is_ident("arg") {
                SchmargsAttribute::Arg(ArgAttribute {
                    short: map
                        .remove("short")
                        .map(|v| v.map(|v| v.unwrap_as_literal())),
                    long: map.remove("long").map(|v| v.map(|v| v.unwrap_as_literal())),
                })
            } else if attr.path().is_ident("schmargs") {
                SchmargsAttribute::TopLevel(TopLevelAttribute {
                    iterates_over: map
                        .remove("iterates_over")
                        .map(|v| v.expect("`iterates_over` expects a type"))
                        .map(|v| v.unwrap_as_ident()),
                    name: map
                        .remove("name")
                        .map(|v| v.expect("`name` expects a type"))
                        .map(|v| v.unwrap_as_literal()),
                })
            } else {
                bail!("Unsupported attribute type");
            };

            if !map.is_empty() {
                bail!("Unknown argument to attribute");
            }

            Ok(return_value)
        }
        syn::Meta::NameValue(ref pair) => {
            assert!(attr.path().is_ident("doc"));
            let syn::Expr::Lit(ref value) = pair.value else {
                bail!("Expected literal attribute value ( i.e. doc comment)");
            };
            let syn::Lit::Str(ref value) = value.lit else {
                bail!("Expected str literal attribute value ( i.e. doc comment)");
            };
            return Ok(SchmargsAttribute::Doc(DocAttribute {
                value: value.value().trim().into(),
            }));
        }
        _ => bail!("Expected name-value pair attribute (i.e. doc comment)"),
    }
}

fn parse_attributes(attrs: &[Attribute]) -> Result<AttributeAggregate> {
    let mut doc = None;
    let mut arg = None;
    let mut top_level = None;
    for attr in attrs {
        match parse_attribute(attr)? {
            SchmargsAttribute::Doc(attr) => {
                if doc.is_some() {
                    bail!("Was not expecting two doc attributes!");
                }
                doc = Some(attr);
            }
            SchmargsAttribute::Arg(attr) => {
                if arg.is_some() {
                    bail!("Was not expecting two arg attributes!");
                }
                arg = Some(attr);
            }
            SchmargsAttribute::TopLevel(attr) => {
                if top_level.is_some() {
                    bail!("Was not expecting two arg attributes!");
                }
                top_level = Some(attr);
            }
        }
    }

    Ok(AttributeAggregate {
        doc: doc.expect("Missing doc attribute"),
        arg,
        top_level,
    })
}

pub fn schmargs_derive_impl(input: DeriveInput) -> Result<proc_macro::TokenStream> {
    let struct_name = input.ident;
    let attributes = parse_attributes(&input.attrs)?;
    let command_name = attributes
        .top_level
        .clone()
        .and_then(|v| v.name.clone())
        .map(|v| quote! {#v})
        .unwrap_or_else(|| {
            quote! {env!("CARGO_PKG_NAME")}
        });
    let description = attributes.doc.value;
    let default_lifetime =
        LifetimeParam::new(Lifetime::new("'__schmargs_lifetime", Span::call_site()));
    let generics = input.generics.clone();
    let lifetime = generics.lifetimes().next().unwrap_or(&default_lifetime);

    let impl_generics = if generics.lt_token.is_some() {
        let generics = crate::utils::copy_generics(
            &generics,
            crate::utils::CopyGenericsBoundOption::WithBounds,
        );
        quote! { < #generics > }
    } else {
        quote! { <#lifetime> }
    };

    let string_type = if let Some(TopLevelAttribute {
        iterates_over: Some(ref iterates_over),
        ..
    }) = attributes.top_level
    {
        quote! { #iterates_over }
    } else {
        quote! { &#lifetime str }
    };

    // Generics without the trait bounds
    let bare_generics = if generics.lt_token.is_some() {
        let inner = crate::utils::copy_generics(
            &generics,
            crate::utils::CopyGenericsBoundOption::WithoutBounds,
        );
        let gen = quote! { < #inner > };
        gen
    } else {
        quote! {}
    };

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.clone(),
        _ => bail!("expected a struct with named fields"),
    };

    let args: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let is_bool = field.ty.span().source_text().unwrap() == "bool";
            let is_option = field.ty.span().source_text().unwrap().starts_with("Option");
            let attr = parse_attributes(&field.attrs).unwrap();
            let ident = field.ident.clone().unwrap().clone();
            Arg {
                is_bool,
                is_option,
                attr,
                ident,
            }
        })
        .collect();

    let help_body = impl_help_body(&args);
    let parse_body = impl_parse_body(&string_type, &args);
    let usage_body = impl_usage_body(&command_name, &args);

    let mut gen = quote! {
        impl #impl_generics ::schmargs::Schmargs<#lifetime> for #struct_name #bare_generics {
            type Item = #string_type;

            const NAME: &'static str = #command_name;
            const USAGE: &'static str = {
                #usage_body
            };
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");
            const DESCRIPTION: &'static str = #description;

            fn write_help_with_min_indent(mut f: impl ::core::fmt::Write, mut min_indent: usize) -> Result<usize, ::core::fmt::Error> {
                #help_body
            }

            fn parse(args: impl ::core::iter::Iterator<Item = #string_type >) -> ::core::result::Result<Self, ::schmargs::SchmargsError<#string_type>> {
                #parse_body
            }
        }
    };

    // Allow showing help with `println!("{args}")
    gen.extend(quote! {
        impl #impl_generics ::core::fmt::Display for #struct_name #bare_generics {
            fn fmt(&self, __schmargs_formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                Self::write_help_with_min_indent(__schmargs_formatter, 0)?;
                Ok(())
            }
        }
    });

    Ok(gen.into())
}

fn impl_parse_body(string_type: &TokenStream, args: &[Arg]) -> TokenStream {
    let mut body = quote! {
        let mut args = ::schmargs::utils::DumbIterator::from_args(args);
    };

    for arg in args {
        let ident = &arg.unique_ident();
        body.extend(match arg.kind() {
            ArgKind::Flag => {
                quote! {
                    #[allow(non_snake_case)]
                    let mut #ident = false;
                }
            }
            ArgKind::Positional | ArgKind::Option => {
                quote! {
                    #[allow(non_snake_case)]
                    let mut #ident = ::schmargs::SchmargsField::<#string_type>::as_option();
                }
            }
        });
    }

    let short_flag_match_body = {
        let mut body: TokenStream = Default::default();
        for arg in args
            .iter()
            .filter(|a| a.kind() == ArgKind::Flag || a.kind() == ArgKind::Option)
        {
            let ident = &arg.unique_ident();
            if let Some(short) = arg.short() {
                body.extend(quote! { #short =>});
                if arg.kind() == ArgKind::Flag {
                    body.extend(quote! { {
                            #ident = true;
                        },
                    });
                } else {
                    body.extend(quote! { {
                                match args.next() {
                                    Some(::schmargs::utils::DumbArgument::Positional(value)) => {
                                        #ident = Some(::schmargs::SchmargsField::<#string_type>::parse_str(value)?);
                                    },
                                    _=> {return Err(::schmargs::SchmargsError::ExpectedValue(stringify!(#ident)));}
                                }
                            },
                        });
                }
            }
        }

        body
    };

    let match_body = {
        let mut body: TokenStream = Default::default();
        for arg in args
            .iter()
            .filter(|a| a.kind() == ArgKind::Flag || a.kind() == ArgKind::Option)
        {
            let ident = &arg.unique_ident();
            if let Some(long) = arg.long() {
                body.extend(
                    quote! { ::schmargs::utils::DumbArgument::LongFlag(__schmargs_throwaway) if ::core::convert::AsRef::<str>::as_ref(&__schmargs_throwaway) == #long =>},
                );
                if arg.kind() == ArgKind::Flag {
                    body.extend(quote! { {
                            #ident = true;
                        },
                    });
                } else {
                    body.extend(quote! { {
                                match args.next() {
                                    Some(::schmargs::utils::DumbArgument::Positional(value)) => {
                                        #ident = Some(::schmargs::SchmargsField::<#string_type>::parse_str(value)?);
                                    },
                                    _=> {return Err(::schmargs::SchmargsError::ExpectedValue(stringify!(#ident)));}
                                }
                            },
                        });
                }
            }
        }

        let (num, positional): (Vec<usize>, Vec<proc_macro2::Ident>) = args
            .iter()
            .filter(|a| a.kind() == ArgKind::Positional)
            .map(|a| a.unique_ident().clone())
            .enumerate()
            .unzip();
        if !positional.is_empty() {
            let (num, positional) = (num.into_iter(), positional.into_iter());
            body.extend(quote! {
                ::schmargs::utils::DumbArgument::Positional(value) => {
                    match pos_count {
                    #(
                        #num => {#positional = Some(::schmargs::SchmargsField::<#string_type>::parse_it(value, (&mut args).map(|v|v.into_inner()))?);},
                    )*
                        _ => {return ::core::result::Result::Err(::schmargs::SchmargsError::UnexpectedValue(value));}
                    }
                    pos_count += 1;
                },
            });
        } else {
            body.extend(quote! {
                ::schmargs::utils::DumbArgument::Positional(val) => {
                    ::core::result::Result::Err(::schmargs::SchmargsError::UnexpectedValue(val))?;
                },
            });
        };
        body.extend(quote! {
            ::schmargs::utils::DumbArgument::LongFlag(val) => {
                ::core::result::Result::Err(::schmargs::SchmargsError::NoSuchLongFlag(val))?;
            }
        });

        body
    };

    let return_body = {
        let mut body: TokenStream = Default::default();

        for arg in args {
            let original_ident = &arg.ident;
            let unique_ident = &arg.unique_ident();
            body.extend(match arg.kind() {
                ArgKind::Flag => quote! {
                    #original_ident: #unique_ident,
                },
                ArgKind::Positional | ArgKind::Option => quote! {
                    #original_ident: #unique_ident.ok_or(
                        ::schmargs::SchmargsError::ExpectedValue(stringify!(#original_ident))
                    )?,
                },
            });
        }

        body
    };

    body.extend(quote! {

        let mut pos_count = 0;

        while let Some(arg) = args.next() {
            match arg {
                ::schmargs::utils::DumbArgument::ShortFlags(shorts) => {
                    for short in AsRef::<str>::as_ref(&shorts).strip_prefix("-").expect("Bug: expected short flag here").chars() {
                        let short: char = short;
                        match short {
                            #short_flag_match_body
                            __schmargs_misc_short_flag => {
                                return ::core::result::Result::Err(
                                    ::schmargs::SchmargsError::NoSuchShortFlag(
                                            __schmargs_misc_short_flag
                                    )
                                );
                            }
                        }
                    }
                },
                #match_body
            }
        }

        Ok(Self {
            #return_body
        })
    });

    body
}

fn impl_help_body(args: &[Arg]) -> TokenStream {
    let mut body = {
        let long = args.iter().map(|v| v.long().unwrap_or_default());
        quote! {
            #(
                min_indent = ::core::cmp::max(min_indent, "-h, ".len() + #long.len() + 1);
            )*
        }
    };

    body.extend(quote! {
        writeln!(f, "{}", Self::DESCRIPTION)?;
        writeln!(f)?;
        writeln!(f, "Usage: {}", Self::USAGE)?;
    });

    if args.iter().any(|v| v.kind() == ArgKind::Positional) {
        body.extend(quote! {
            writeln!(f)?;
            writeln!(f, "Arguments:")?;
        });
        for arg in args.iter().filter(|v| v.kind() == ArgKind::Positional) {
            let ident = &arg.ident;
            let desc = &arg.attr.doc.value;
            body.extend(quote! {
                writeln!(f, "[{}]        {}", stringify!(#ident), #desc)?;
            });
        }
    }

    if args
        .iter()
        .any(|v| v.kind() == ArgKind::Flag || v.kind() == ArgKind::Option)
    {
        body.extend(quote! {
            writeln!(f)?;
            write!(f, "Options:")?;
        });
        for arg in args
            .iter()
            .filter(|v| v.kind() == ArgKind::Flag || v.kind() == ArgKind::Option)
        {
            let desc = &arg.attr.doc.value;

            body.extend(quote! {
                let mut revindent = 0;
                writeln!(f)?;
            });

            if let Some(short) = arg.short() {
                body.extend(quote! {
                    write!(f, "-{}", #short)?;
                    revindent += 2;
                });
                if arg.long().is_some() {
                    body.extend(quote! {
                        write!(f, ", ")?;
                        revindent += 2;
                    });
                }
            }
            if let Some(long) = arg.long() {
                body.extend(quote! {
                    write!(f, #long)?;
                    revindent += #long.len();
                });
            }

            body.extend(quote! {
                for _ in 0..min_indent.saturating_sub(revindent) {
                    write!(f, " ")?;
                }
                write!(f, "{}", #desc)?;
            });
        }
    }

    body.extend(quote! {
        Ok(min_indent)
    });
    body
}

fn impl_usage_body(command_name: &TokenStream, args: &[Arg]) -> TokenStream {
    let mut body = quote! {};

    if args.iter().any(|v| v.kind() == ArgKind::Flag) {
        body.extend(quote! {
            , " [OPTIONS]"
        });
    }

    for arg in args.iter().filter(|v| v.kind() == ArgKind::Positional) {
        let ident = &arg.ident;
        if arg.is_option {
            body.extend(quote! {
                ," [", stringify!(#ident) ,"]"
            });
        } else {
            body.extend(quote! {
                , " ", stringify!(#ident)
            });
        }
    }

    quote! {
        concat!(#command_name #body)
    }
}
