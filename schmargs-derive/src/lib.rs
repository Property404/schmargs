use proc_macro::TokenStream;
use quote::quote;
use syn::{self, spanned::Spanned, Fields, DataStruct, Data, parse_macro_input, DeriveInput};

#[proc_macro_derive(Schmargs)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;


    let fields = match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };


    let arg_flag = fields.iter().filter(|field|field.ty.span().source_text().unwrap() == "bool").map(|field| &field.ident);
    let arg_positional = fields.iter().filter(|field|field.ty.span().source_text().unwrap() != "bool").map(|field| &field.ident);

    
    let arg_flag2 = arg_flag.clone();
    let arg_positional2 = arg_positional.clone();

    let arg_flag3 = arg_flag.clone();
    let arg_positional3 = arg_positional.clone();

    let num = arg_positional3.clone().enumerate().map(|(i,_)|i);


    let gen = quote! {
        impl<'a> ::schmargs::Schmargs<'a> for #name {
            fn description() -> &'static str {
                unimplemented!("Fuck me")
            }

            fn parse(args: impl Iterator<Item =  &'a str>) -> Result<Self, ::schmargs::SchmargsError> {
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
                                #num => {#arg_positional2 = Some(value.parse().unwrap());},
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
