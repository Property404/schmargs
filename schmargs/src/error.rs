use core::num::ParseIntError;
use crate::Argument;

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

