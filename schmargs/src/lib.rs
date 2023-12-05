#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]
//! A argument parser that can be used with `#[no_std]`
//!
//! # Features
//!
//! * `clap-derive`-inspired derive macro
//! * `#![no_std]`-friendly
//! * Optional arguments
//! * Multi-arg positional arguments and options with [std::vec::Vec]
//! * Custom and default short and long flags
//! * A [wrapper](ArgsWithHelp) that allows for `--help` functionality
//!
//! # Todo
//!
//! * Make sure idents created by proc macro are reasonably unique
//! * Improve documentation
//! * Add usage text
//! * Improve and write tests for help formatting
//!
//! # Helper Attributes
//!
//! ## `schmargs`
//!
//! This is an optional attribute that should be specified at the top level.
//!
//! Arguments:
//!
//! * `name=<str literal>` - The name of the program. Defaults to the crate name.
//! * `iterates_over=<type>` - The string type that's being iterated over. This should be the `Item`
//!  associated type of the [core::iter::Iterator] type passed to [Schmargs::parse]. This defaults
//!  to `&str` with an appropriate lifetime. If you're in an `std` environment and plan on parsing
//!  arguments passed to your program with `Schmargs::parse_env`, `iterates_over` MUST be specified.
//!
//! ## `args`
//!
//! This is an optional attribute that should be specified on an argument.
//!
//! Arguments:
//!
//! * `short[=<char literal>]` - The short flag of the argument. If no value is provided, it will
//!  default to the first letter of the argument name.
//! * `long[=<str literal>]` - The long flag of the argument. If no value is provided, it will
//!  default to the the argument name.
//!
//! # Example
//!
//! When using in an `std` environment, you generally want to specify `iterates_over` to be
//! `String`, so you can iterate over [std::env::Args].
//!
//! ```no_run
//! use schmargs::Schmargs;
//!
//! /// A program to yell at a cloud
//! #[derive(Schmargs)]
//! #[schmargs(iterates_over=String)]
//! struct Args {
//!     /// Yell volume, in decibels
//!     #[arg(short, long)]
//!     volume: Option<u64>,
//!     /// Yell length, in nanoseconds
//!     #[arg(short, long)]
//!     length: Option<u64>,
//!     /// Obscenities to yell
//!     content: Vec<String>,
//! }
//!
//! // This parses the arguments passed to the program
//! let args = Args::parse_env();
//! println!("{:?}", args.content);
//! ```
//!
//! # `#![no_std]` Examples
//!
//! ```
//! use schmargs::Schmargs;
//!
//! /// A simple memory dump program
//! #[derive(Schmargs)]
//! #[schmargs(name = "hexdump")]
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
//! #[schmargs(name = "greet")]
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

mod error;
#[doc(hidden)]
pub mod utils;
mod wrappers;

pub use error::SchmargsError;
pub use schmargs_derive::*;
pub use wrappers::{ArgsWithHelp, ArgsWithVersion};

use core::fmt;

/// A field that can be parsed by Schmargs
pub trait SchmargsField<T>: Sized {
    /// Construct type from string
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>>;
    /// Construct type from iterator
    fn parse_it(val: T, _it: impl Iterator<Item = T>) -> Result<Self, SchmargsError<T>> {
        Self::parse_str(val)
    }
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

#[doc(hidden)]
pub trait StringLike: AsRef<str> {}
impl StringLike for str {}
impl StringLike for &str {}
#[cfg(feature = "std")]
impl StringLike for String {}
#[cfg(feature = "std")]
impl StringLike for &String {}

impl<T: StringLike> SchmargsField<T> for T {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        Ok(val)
    }
}

#[cfg(feature = "std")]
impl SchmargsField<String> for Vec<String> {
    fn parse_str(val: String) -> Result<Self, SchmargsError<String>> {
        let mut vec = Vec::with_capacity(1);
        for val in val.split(',') {
            vec.push(val.into());
        }
        Ok(vec)
    }

    fn parse_it(
        val: String,
        it: impl Iterator<Item = String>,
    ) -> Result<Self, SchmargsError<String>> {
        let hint = it.size_hint();
        let mut vec = Vec::with_capacity(1 + hint.1.unwrap_or(hint.0));
        vec.push(val);
        for val in it {
            vec.push(val);
        }
        Ok(vec)
    }
}

impl<U, T: SchmargsField<U>> SchmargsField<U> for Option<T> {
    fn parse_str(val: U) -> Result<Self, SchmargsError<U>> {
        Ok(Some(T::parse_str(val)?))
    }

    fn as_option() -> Option<Self> {
        Some(None)
    }
}

/// An argument parser
pub trait Schmargs<'a>: Sized {
    /// The item [Schmargs::parse] will iterate over. This is typically &str or [String]
    type Item;

    /// Name of the command
    const NAME: &'static str;

    /// Command version
    const VERSION: &'static str;

    /// Command description
    const DESCRIPTION: &'static str;

    /// Write help text to `f`
    /// Returns the indent used, which will be greater than or equal to `min_indent`
    ///
    /// Unless you're implementing [Schmargs], you most likely want to use the
    /// [Display](core::fmt::Display) impl:
    ///
    /// ```
    /// use schmargs::Schmargs;
    ///
    /// /// Fake program
    /// #[derive(Schmargs)]
    /// struct Args {}
    ///
    /// let args = Args::parse("".split_whitespace()).unwrap();
    ///
    /// println!("{args}");
    /// ```
    fn write_help_with_min_indent(
        f: impl fmt::Write,
        min_indent: usize,
    ) -> Result<usize, fmt::Error>;

    /// Construct from an iterator of arguments
    fn parse(args: impl Iterator<Item = Self::Item>) -> Result<Self, SchmargsError<Self::Item>>;

    /// Convenience function to parse from [std::env::args]
    ///
    /// Note that this will exit the program on error. If this is not the behavior you want, use
    /// [Schmargs::parse]
    ///
    /// Must be used with `#[schmargs(iterates_over=String)]`
    #[cfg(feature = "std")]
    fn parse_env() -> Self
    where
        Self::Item: From<String> + fmt::Display,
    {
        let args = std::env::args().skip(1).map(Into::into);

        match Self::parse(args) {
            Ok(args) => args,
            Err(err) => {
                eprintln!("{}: error: {err}", Self::NAME);
                std::process::exit(1);
            }
        }
    }
}
