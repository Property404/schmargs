use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    self, parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields, Lifetime,
    LifetimeParam,
};

#[proc_macro_derive(Schmargs)]
pub fn schmargs_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
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
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let arg_flag = fields
        .iter()
        .filter(|field| field.ty.span().source_text().unwrap() == "bool")
        .map(|field| &field.ident);
    let arg_positional = fields
        .iter()
        .filter(|field| field.ty.span().source_text().unwrap() != "bool")
        .map(|field| &field.ident);

    let arg_flag2 = arg_flag.clone();
    let arg_positional2 = arg_positional.clone();

    let arg_flag3 = arg_flag.clone();
    let arg_positional3 = arg_positional.clone();

    let num = arg_positional3.clone().enumerate().map(|(i, _)| i);

    //println!("IMPL: {:?}", pretty_print(&impl_generics));
    //println!("LIFETIME: {:?}", &quote! {#lifetime});
    println!("STRUCT: {:?}", quote! {#struct_generics});

    let gen = quote! {
        impl #impl_generics ::schmargs::Schmargs <#lifetime> for #name #struct_generics {
            fn description() -> &'static str {
                unimplemented!("Fuck me")
            }

            fn parse(args: impl Iterator<Item =  & #lifetime str>) -> Result<Self, ::schmargs::SchmargsError> {
                let args = ::schmargs::ArgumentIterator::from_args(args);

                // flags
                #(
                    let mut #arg_flag = false;
                )*

                // positionasl
                #(
                    let mut #arg_positional = ::core::option::Option::None;
                )*

                let mut pos_count = 0;

                for arg in args {
                    match arg {
                        #(
                            ::schmargs::Argument::LongFlag(stringify!(#arg_flag2)) => {
                                #arg_flag2 = true;
                            },
                        )*
                        ::schmargs::Argument::Positional(value) => {
                            match pos_count {
                            #(
                                #num => {#arg_positional2 = Some(::schmargs::SchmargsField::parse_str(value)?);},
                            )*
                                _ => {return Err(::schmargs::SchmargsError::TooManyArguments);}
                            }
                            pos_count += 1;
                        },
                        _=> {return Err(::schmargs::SchmargsError::ParseError);}
                    }
                }

                Ok(Self {
                    // flags
                    #(
                        #arg_flag3,
                    )*
                    // positionals
                    #(
                        #arg_positional3: #arg_positional3.unwrap(),
                    )*
                })
            }
        }
    };

    gen.into()
}
