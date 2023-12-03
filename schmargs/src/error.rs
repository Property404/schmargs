use crate::Argument;
use core::{
    fmt::{self, Display},
    num::ParseIntError,
};

#[derive(Debug)]
pub enum SchmargsError<T: AsRef<str>> {
    ParseInt(ParseIntError),
    NoSuchOption(Argument<T>),
    UnexpectedValue(T),
    ExpectedValue(&'static str),
    NoZerothArgument,
}

impl<T: AsRef<str>> From<ParseIntError> for SchmargsError<T> {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl<T: AsRef<str> + fmt::Debug + fmt::Display> Display for SchmargsError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ParseInt(err) => err.fmt(f),
            Self::NoSuchOption(val) => {
                write!(f, "No such argument: {val:?}")
            }
            Self::UnexpectedValue(val) => {
                write!(f, "Did not expect positional value: {val}")
            }
            Self::ExpectedValue(val) => {
                write!(f, "Expected value for {val}")
            }
            Self::NoZerothArgument => {
                write!(f, "No zeroth argument")
            }
        }
    }
}

#[cfg(feature = "std")]
impl<T: AsRef<str> + fmt::Debug + fmt::Display> std::error::Error for SchmargsError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseInt(err) => Some(err),
            _ => None,
        }
    }
}
