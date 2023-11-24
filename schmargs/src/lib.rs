#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
//! A `#![no_std]` argument parser
//!
//! # Example
//! This example was stolen from [Clap](https://docs.rs/clap/latest/clap/)
//! ```
//! use schmargs::Schmargs;
//!
//! #[derive(Schmargs)]
//! /// A simple program to greet a person
//! struct Args<'a> {
//!     /// Name of the person to greet
//!     #[arg(short, long)]
//!     name: &'a str,
//!
//!     /// Number of times to greet
//!     count: u8
//! }
//!
//! let args = Args::parse("--name me 4".split_whitespace()).unwrap();
//! assert_eq!(args.name, "me");
//! assert_eq!(args.count, 4);
//!
//! ```
//!

pub mod utils;

pub use schmargs_derive::*;

use core::num::ParseIntError;
use utils::Argument;

/// A field that can be parsed by Schmargs
pub trait SchmargsField<'a>: Sized {
    /// Construct type from string
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>>;
    // Mechanism used to make `Option` types optional
    #[doc(hidden)]
    fn as_option() -> Option<Self> {
        None
    }
}

macro_rules! impl_on_integer {
    ($ty:ty) => {
        impl<'a> SchmargsField<'a> for $ty {
            fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>> {
                if let Some(val) = val.strip_prefix("0x") {
                    Ok(<$ty>::from_str_radix(val, 16)?)
                } else {
                    Ok(val.parse()?)
                }
            }
        }
    };
}

impl_on_integer!(u8);
impl_on_integer!(u16);
impl_on_integer!(u32);
impl_on_integer!(u64);
impl_on_integer!(u128);
impl_on_integer!(usize);
impl_on_integer!(i8);
impl_on_integer!(i16);
impl_on_integer!(i32);
impl_on_integer!(i64);
impl_on_integer!(i128);
impl_on_integer!(isize);

impl<'a> SchmargsField<'a> for &'a str {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>> {
        Ok(val)
    }
}

impl<'a, T: SchmargsField<'a>> SchmargsField<'a> for Option<T> {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>> {
        Ok(Some(T::parse_str(val)?))
    }

    fn as_option() -> Option<Self> {
        Some(None)
    }
}

#[derive(Debug)]
pub enum SchmargsError<'a> {
    ParseInt(ParseIntError),
    NoSuchOption(Argument<'a>),
    TooManyArguments,
    ExpectedValue(&'static str),
}

impl<'a> From<ParseIntError> for SchmargsError<'a> {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

/// An argument parser
pub trait Schmargs<'a>: Sized {
    /// Get command description
    fn description() -> &'static str;
    /// Construct from an iterator of argument
    fn parse(args: impl Iterator<Item = &'a str>) -> Result<Self, SchmargsError<'a>>;
}

pub enum ArgsWithHelp<T: for<'a> Schmargs<'a>> {
    Help,
    Args(T),
}

impl<'a, T: for<'b> Schmargs<'b>> Schmargs<'a> for ArgsWithHelp<T> {
    fn description() -> &'static str {
        T::description()
    }

    fn parse(args: impl Iterator<Item = &'a str>) -> Result<Self, SchmargsError<'a>> {
        match T::parse(args) {
            Err(SchmargsError::NoSuchOption(Argument::LongFlag("help")))
            | Err(SchmargsError::NoSuchOption(Argument::ShortFlag('h'))) => Ok(Self::Help),
            Ok(other) => Ok(Self::Args(other)),
            Err(other) => Err(other),
        }
    }
}
