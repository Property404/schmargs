use proc_macro::{Span, TokenStream};
use proc_macro2::{Literal, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::{
    self, parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields,
    Lifetime, LifetimeParam,
};

#[derive(Debug, Clone)]
enum SchmargsAttribute {
    ArgAttribute(ArgAttribute),
    DocAttribute(DocAttribute),
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
    ident: proc_macro2::Ident,
    is_bool: bool,
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

    fn long(&self) -> Option<Literal> {
        if let Some(ArgAttribute {
            long: Some(long), ..
        }) = &self.attr.arg
        {
            return Some(
                long.clone()
                    .unwrap_or_else(|| Literal::string(&self.ident.to_string())),
            );
        }
        None
    }
}

fn parse_attribute(attr: &Attribute) -> SchmargsAttribute {
    match attr.meta {
        syn::Meta::List(ref list) => {
            assert!(attr.path().is_ident("arg"));

            let tokens = list.parse_args::<proc_macro2::TokenStream>().unwrap();

            let mut map = HashMap::new();

            let mut key = None;
            for token in tokens {
                match token {
                    TokenTree::Ident(ident) => {
                        // Later we can remove this assert and add the ident as a value
                        assert!(key.is_none(), "Cannot use identifier as value in attribute");

                        key = Some(ident.to_string());
                    }
                    TokenTree::Punct(punct) => match punct.as_char() {
                        ',' => {
                            if let Some(key) = key.take() {
                                map.insert(key, None);
                            }
                        }
                        '=' => {}
                        _ => {
                            panic!("Unexpected punctuation in attribute");
                        }
                    },
                    TokenTree::Literal(literal) => {
                        if let Some(key) = key.take() {
                            map.insert(key, Some(literal));
                        } else {
                            panic!("Unexpected literal in attribute")
                        }
                    }
                    TokenTree::Group(_) => panic!("Unexpected token tree type"),
                }
            }
            if let Some(key) = key.take() {
                map.insert(key, None);
            }

            let return_value = SchmargsAttribute::ArgAttribute(ArgAttribute {
                short: map.remove("short"),
                long: map.remove("long"),
            });

            if !map.is_empty() {
                panic!("Unknown argument to 'arg' attribute");
            }

            return_value
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

    let args: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let is_bool = field.ty.span().source_text().unwrap() == "bool";
            let attr = parse_attributes(&field.attrs);
            let ident = field.ident.clone().unwrap().clone();
            Arg {
                is_bool,
                attr,
                ident,
            }
        })
        .collect();

    let help_body = impl_help_body(&args);
    let parse_body = impl_parse_body(&args);

    let gen = quote! {
        impl #impl_generics ::schmargs::Schmargs <#lifetime> for #name #struct_generics {
            fn description() -> &'static str {
                #description
            }

            fn write_help_with_min_indent(mut f: impl ::core::fmt::Write, name: impl ::core::convert::AsRef<str>, mut min_indent: usize) -> Result<usize, ::core::fmt::Error> {
                #help_body
            }

            fn parse(args: impl ::core::iter::Iterator<Item =  & #lifetime str>) -> ::core::result::Result<Self, ::schmargs::SchmargsError<#lifetime>> {
                #parse_body
            }
        }
    };

    gen.into()
}

fn impl_parse_body(args: &[Arg]) -> proc_macro2::TokenStream {
    let mut body = quote! {
        let mut args = ::schmargs::utils::ArgumentIterator::from_args(args);
    };

    for arg in args {
        let ident = &arg.ident;
        body.extend(match arg.kind() {
            ArgKind::Flag => {
                quote! {
                    let mut #ident = false;
                }
            }
            ArgKind::Positional | ArgKind::Option => {
                quote! {
                    let mut #ident = ::schmargs::SchmargsField::as_option();
                }
            }
        });
    }

    let match_body = {
        let mut body: proc_macro2::TokenStream = Default::default();
        for arg in args
            .iter()
            .filter(|a| a.kind() == ArgKind::Flag || a.kind() == ArgKind::Option)
        {
            let ident = &arg.ident;
            if let Some(short) = arg.short() {
                body.extend(quote! { ::schmargs::utils::Argument::ShortFlag(#short) =>});
                if arg.kind() == ArgKind::Flag {
                    body.extend(quote! { {
                            #ident = true;
                        },
                    });
                } else {
                    body.extend(quote! { {
                                match args.next() {
                                    Some(::schmargs::utils::Argument::Positional(value)) => {
                                        #ident = Some(::schmargs::SchmargsField::parse_str(value)?);
                                    },
                                    _=> {return Err(::schmargs::SchmargsError::ExpectedValue(stringify!(#ident)));}
                                }
                            },
                        });
                }
            }

            if let Some(long) = arg.long() {
                body.extend(quote! { ::schmargs::utils::Argument::LongFlag(#long) =>});
                if arg.kind() == ArgKind::Flag {
                    body.extend(quote! { {
                            #ident = true;
                        },
                    });
                } else {
                    body.extend(quote! { {
                                match args.next() {
                                    Some(::schmargs::utils::Argument::Positional(value)) => {
                                        #ident = Some(::schmargs::SchmargsField::parse_str(value)?);
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
            .map(|a| a.ident.clone())
            .enumerate()
            .unzip();
        if !positional.is_empty() {
            let (num, positional) = (num.into_iter(), positional.into_iter());
            body.extend(quote! {
                ::schmargs::utils::Argument::Positional(value) => {
                    match pos_count {
                    #(
                        #num => {#positional = Some(::schmargs::SchmargsField::parse_str(value)?);},
                    )*
                        _ => {return ::core::result::Result::Err(::schmargs::SchmargsError::UnexpectedValue(value));}
                    }
                    pos_count += 1;
                },
            });
        } else {
            body.extend(quote! {
                ::schmargs::utils::Argument::Positional(val) => {
                    ::core::result::Result::Err(::schmargs::SchmargsError::UnexpectedValue(val))?;
                },
            });
        };
        body.extend(quote! {
            arg => {::core::result::Result::Err(::schmargs::SchmargsError::NoSuchOption(arg))?;}
        });

        body
    };

    let return_body = {
        let mut body: proc_macro2::TokenStream = Default::default();

        for arg in args {
            let ident = &arg.ident;
            body.extend(match arg.kind() {
                ArgKind::Flag => quote! {
                    #ident,
                },
                ArgKind::Positional | ArgKind::Option => quote! {
                    #ident: #ident.ok_or(
                        ::schmargs::SchmargsError::ExpectedValue(stringify!(#ident))
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
                #match_body
            }
        }

        Ok(Self {
            #return_body
        })
    });

    body
}

fn impl_help_body(args: &[Arg]) -> proc_macro2::TokenStream {
    let mut body = {
        let ident = args.iter().map(|v| &v.ident);
        quote! {
            #(
                min_indent = ::core::cmp::max(min_indent, "-h, --".len() + stringify!(#ident).len() + 1);
            )*
        }
    };

    body.extend(quote! {
        writeln!(f, "{}", Self::description())?;
        writeln!(f)?;
        write!(f, "Usage: {}", name.as_ref())?;
    });

    if args.iter().any(|v| v.kind() == ArgKind::Flag) {
        body.extend(quote! {
            write!(f, " [OPTIONS]")?;
        });
    }

    for arg in args.iter().filter(|v| v.kind() == ArgKind::Positional) {
        let ident = &arg.ident;
        body.extend(quote! {
            write!(f, " [{}]", stringify!(#ident))?;
        });
    }
    body.extend(quote! {
        writeln!(f)?;
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
                    write!(f, "--{}", #long)?;
                    revindent += 2 + #long.len();
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
