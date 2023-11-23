#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
use core::num::ParseIntError;

pub trait SchmargsField<'a>: Sized {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>>;
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

#[derive(Debug)]
pub enum SchmargsError<'a> {
    ParseInt(ParseIntError),
    NoSuchOption(Argument<'a>),
    TooManyArguments,
    NotEnoughArguments,
}

impl<'a> From<ParseIntError> for SchmargsError<'a> {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Argument<'a> {
    ShortFlag(char),
    LongFlag(&'a str),
    Positional(&'a str),
}

pub struct ArgumentIterator<'a, I: Iterator<Item = &'a str>> {
    hit_double_dash: bool,
    shortflags: Option<core::str::Chars<'a>>,
    args: I,
}

impl<'a, I: Iterator<Item = &'a str>> ArgumentIterator<'a, I> {
    pub fn from_args(args: I) -> Self {
        Self {
            hit_double_dash: false,
            shortflags: None,
            args,
        }
    }
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for ArgumentIterator<'a, I> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Argument<'a>> {
        if let Some(ref mut shortflags) = &mut self.shortflags {
            if let Some(flag) = shortflags.next() {
                return Some(Argument::ShortFlag(flag));
            } else {
                self.shortflags = None;
            }
        }

        let Some(arg) = self.args.next() else {
            return None;
        };

        if self.hit_double_dash {
            return Some(Argument::Positional(arg));
        }

        if let Some(arg) = arg.strip_prefix("--") {
            if arg.is_empty() {
                self.hit_double_dash = true;
                return self.next();
            }
            Some(Argument::LongFlag(arg))
        } else if let Some(flags) = arg.strip_prefix('-') {
            self.shortflags = Some(flags.chars());
            return self.next();
        } else {
            Some(Argument::Positional(arg))
        }
    }
}

pub trait Schmargs<'a>: Sized {
    fn description() -> &'static str;
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
