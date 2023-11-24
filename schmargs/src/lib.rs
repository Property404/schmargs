#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
//! A `#![no_std]` argument parser
//!
//! # Example
//!
//! ```
//! use schmargs::Schmargs;
//!
//! #[derive(Schmargs)]
//! /// A simple memory dump program
//! struct Args {
//!     /// Show color
//!     #[arg(short, long)]
//!     color: bool,
//!     /// Disable sanity checks
//!     #[arg(short='f', long="force")]
//!     no_null_check: bool,
//!     /// How many bytes to show per line
//!     #[arg(short, long)]
//!     group: Option<u8>, // this is optional
//!     /// Starting memory address
//!     start: usize, // required positional argument
//!     /// Number of bytes to read
//!     len: usize // required positional argument
//! }
//!
//! let args = Args::parse("-f --group 8 0x40000000 256".split_whitespace()).unwrap();
//! assert_eq!(args.color, false);
//! assert_eq!(args.no_null_check, true);
//! assert_eq!(args.group, Some(8));
//! assert_eq!(args.start, 0x40000000);
//! assert_eq!(args.len, 256);
//!
//! ```
//!
//! When strings are involved, you need to add a generic lifetime parameter
//!
//! ```
//! use schmargs::Schmargs;
//!
//! /// A very important program to greet somebody
//! #[derive(Schmargs)]
//! struct Args <'a> {
//!     /// Should we kick the person's shins after greeting them?
//!     #[arg(short,long="kick")]
//!     kick_shins: bool,
//!     /// The person to greet
//!     person: &'a str
//! }
//!
//! let args = Args::parse("Dagan".split_whitespace()).unwrap();
//! assert_eq!(args.kick_shins, false);
//! assert_eq!(args.person, "Dagan");
//! ```

pub mod utils;

pub use schmargs_derive::*;

use core::fmt;
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
    UnexpectedValue(&'a str),
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

    fn write_help_with_min_indent(
        mut f: impl fmt::Write,
        name: impl AsRef<str>,
        min_indent: usize,
    ) -> Result<usize, fmt::Error> {
        let prefix = "-h, --help";
        let min_indent = core::cmp::max(min_indent, prefix.len() + 1);
        let min_indent = core::cmp::max(
            min_indent,
            T::write_help_with_min_indent(&mut f, name, min_indent)?,
        );
        writeln!(f)?;
        write!(f, "{}", prefix)?;
        for _ in 0..(min_indent - prefix.len()) {
            write!(f, " ")?;
        }
        write!(f, "Print help")?;
        Ok(min_indent)
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
