#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]
//! A `#![no_std]` argument parser
//!
//! # Example
//!
//! ```
//! use schmargs::Schmargs;
//!
//! /// A simple memory dump program
//! #[derive(Schmargs)]
//! struct Args {
//!     /// Show color
//!     #[arg(short, long)]
//!     color: bool,
//!     /// Disable sanity checks
//!     #[arg(short = 'f', long = "force")]
//!     no_null_check: bool,
//!     /// How many bytes to show per line
//!     #[arg(short, long)]
//!     group: Option<u8>, // this is optional
//!     /// Starting memory address
//!     start: usize, // required positional argument
//!     /// Number of bytes to read
//!     len: usize, // required positional argument
//! }
//!
//! let args = Args::parse("-f --group 8 0x40000000 256".split_whitespace()).unwrap();
//! assert_eq!(args.color, false);
//! assert_eq!(args.no_null_check, true);
//! assert_eq!(args.group, Some(8));
//! assert_eq!(args.start, 0x40000000);
//! assert_eq!(args.len, 256);
//! ```
//!
//! When strings are involved, you need to add a generic lifetime parameter
//!
//! ```
//! use schmargs::Schmargs;
//!
//! /// A very important program to greet somebody
//! #[derive(Schmargs)]
//! struct Args<'a> {
//!     /// Should we kick the person's shins after greeting them?
//!     #[arg(short, long = "kick")]
//!     kick_shins: bool,
//!     /// The person to greet
//!     person: &'a str,
//! }
//!
//! let args = Args::parse("Dagan".split_whitespace()).unwrap();
//! assert_eq!(args.kick_shins, false);
//! assert_eq!(args.person, "Dagan");
//! ```

#[doc(hidden)]
pub mod utils;
pub mod wrappers;

pub use schmargs_derive::*;

use core::fmt;
use core::num::ParseIntError;

#[derive(Debug, PartialEq, Eq)]
pub enum Argument<T: AsRef<str>> {
    ShortFlag(char),
    LongFlag(T),
    Positional(T),
}

/// A field that can be parsed by Schmargs
pub trait SchmargsField<T: AsRef<str>>: Sized {
    /// Construct type from string
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>>;
    // Mechanism used to make `Option` types optional
    #[doc(hidden)]
    fn as_option() -> Option<Self> {
        None
    }
}

macro_rules! impl_on_integer {
    ($ty:ty) => {
        impl<T: AsRef<str>> SchmargsField<T> for $ty {
            fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
                let val = val.as_ref();
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

// Needed because other implementers might implement AsRef<str> in the future
#[doc(hidden)]
pub trait BullshitTrait {}
impl BullshitTrait for &str {}
#[cfg(feature = "std")]
impl BullshitTrait for &String {}
#[cfg(feature = "std")]
impl BullshitTrait for String {}

impl<T: AsRef<str> + BullshitTrait> SchmargsField<T> for T {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        Ok(val)
    }
}

impl<U: AsRef<str>, T: SchmargsField<U>> SchmargsField<U> for Option<T> {
    fn parse_str(val: U) -> Result<Self, SchmargsError<U>> {
        Ok(Some(T::parse_str(val)?))
    }

    fn as_option() -> Option<Self> {
        Some(None)
    }
}

#[derive(Debug)]
pub enum SchmargsError<T: AsRef<str>> {
    ParseInt(ParseIntError),
    NoSuchOption(Argument<T>),
    UnexpectedValue(T),
    ExpectedValue(&'static str),
}

impl<T: AsRef<str>> From<ParseIntError> for SchmargsError<T> {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

/// An argument parser
pub trait Schmargs<T: AsRef<str>>: Sized {
    /// Get command description
    fn description() -> &'static str;
    /// Write help text to `f`
    /// Returns the indent used, which will be greater than or equal to `min_indent`
    fn write_help_with_min_indent(
        f: impl fmt::Write,
        name: impl AsRef<str>,
        min_indent: usize,
    ) -> Result<usize, fmt::Error>;
    /// Write help text to `f`
    fn write_help(f: impl fmt::Write, name: impl AsRef<str>) -> fmt::Result {
        Self::write_help_with_min_indent(f, name, 0)?;
        Ok(())
    }
    /// Construct from an iterator of argument
    fn parse(args: impl Iterator<Item = T>) -> Result<Self, SchmargsError<T>>;
}
