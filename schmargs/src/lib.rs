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

mod error;
#[doc(hidden)]
pub mod utils;
mod wrappers;

pub use error::SchmargsError;
pub use schmargs_derive::*;
pub use wrappers::ArgsWithHelp;

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

    /// Convenience function to parse from [std::env::args]
    ///
    /// Must be used with `#[schmargs(iterates_over=String)]`
    ///
    /// Returns a tuple of the command name, and the parse arguments
    #[cfg(feature = "std")]
    fn parse_env() -> Self
    where
        T: From<String> + fmt::Display,
    {
        let mut args = std::env::args();
        let Some(command) = args.next() else {
            eprintln!("No arguments");
            std::process::exit(1);
        };

        let args = args.map(|v| v.into());
        match Self::parse(args) {
            Ok(args) => args,
            Err(err) => {
                eprintln!("{command}: error: {err}");
                std::process::exit(1);
            }
        }
    }
}
