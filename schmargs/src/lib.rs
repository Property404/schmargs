#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
use core::num::ParseIntError;

pub trait SchmargsField<'a>: Sized {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>>;
}

macro_rules! impl_for_unsigned_int {
    ($ty:ty) => {
        impl<'a> SchmargsField<'a> for $ty {
            fn parse_str(val: &'a str) -> Result<Self, SchmargsError<'a>>
            {
                if let Some(val) = val.strip_prefix("0x") {
                    Ok(<$ty>::from_str_radix(val, 16)?)
                } else {
                    Ok(<$ty>::from_str_radix(val, 10)?)
                }
 
            }
        }
    }
}
impl_for_unsigned_int!(u8);
impl_for_unsigned_int!(u16);
impl_for_unsigned_int!(u32);
impl_for_unsigned_int!(u64);
impl_for_unsigned_int!(u128);
impl_for_unsigned_int!(usize);

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
    Positional(&'a str)
}

pub struct ArgumentIterator<'a, I: Iterator<Item = &'a str>> {
    hit_double_dash: bool,
    shortflags: Option<core::str::Chars<'a>>,
    args: I
}

impl<'a, I: Iterator<Item = &'a str>>  ArgumentIterator<'a, I> {
    pub fn from_args(args: I) -> Self {
        Self {
            hit_double_dash: false,
            shortflags: None,
            args
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
    fn description() -> &'static str {
        "Description to be written"
    }
    fn parse(args: impl Iterator<Item =  &'a str>) -> Result<Self, SchmargsError<'a>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo<'a> {
        bar: bool,
        baz: u64,
        biz: &'a str
    }

    impl<'a> Schmargs<'a> for Foo<'a> {
        fn parse(args: impl Iterator<Item =  &'a str>) -> Result<Self, SchmargsError<'a>> {
            let args = ArgumentIterator::from_args(args);

            // flags
            let mut bar = false;

            // positionals
            let mut baz = None;
            let mut biz = None;
            let mut pos_count = 0;

            for arg in args {
                match arg {
                    Argument::LongFlag("bar") => {
                        bar = true;
                    },
                    Argument::Positional(pos) => {
                        match pos_count {
                            0 => {baz = Some(pos.parse().unwrap());}
                            1 => {biz = Some(pos.try_into().unwrap());}
                            _ => panic!("TOO MANY ARGS")
                        };
                    },
                    Argument::ShortFlag(c) => {
                        panic!("Unexpected flag: {c}");
                    }
                    _=> todo!("HM")
                }
            }

            Ok(Self {
                // flags
                bar: bar,
                // positionals
                baz: baz.unwrap(),
                biz: biz.unwrap()
            })
        }
    }

    #[test]
    fn check_iteration() {
        let mut it = ArgumentIterator::from_args("-to part --long x -- --wee -xdf".split_whitespace());
        assert_eq!(it.next().unwrap(), Argument::ShortFlag('t'));
        assert_eq!(it.next().unwrap(), Argument::ShortFlag('o'));
        assert_eq!(it.next().unwrap(), Argument::Positional("part"));
        assert_eq!(it.next().unwrap(), Argument::LongFlag("long"));
        assert_eq!(it.next().unwrap(), Argument::Positional("x"));
        // These are following the `--`, so they're taken literally
        assert_eq!(it.next().unwrap(), Argument::Positional("--wee"));
        assert_eq!(it.next().unwrap(), Argument::Positional("-xdf"));
        assert!(it.next().is_none());
    }
}
