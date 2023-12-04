use core::{
    fmt::{self, Display},
    num::ParseIntError,
};

/// The error type used in this crate
#[derive(Debug)]
pub enum SchmargsError<T> {
    /// Transparent wrapper around [ParseIntError]
    ParseInt(ParseIntError),
    /// Passed a short flag that doesn't exist
    NoSuchShortFlag(char),
    /// Passed a long flag that doesn't exist
    NoSuchLongFlag(T),
    /// Did not expect this value
    UnexpectedValue(T),
    /// Expected a value to an argument
    ExpectedValue(&'static str),
    /// Expected a zeroth argument - i.e the command name
    NoZerothArgument,
}

impl<T> From<ParseIntError> for SchmargsError<T> {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl<T: fmt::Display> Display for SchmargsError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ParseInt(err) => err.fmt(f),
            Self::NoSuchShortFlag(val) => {
                write!(f, "'-{val}' is not a valid option")
            }
            Self::NoSuchLongFlag(val) => {
                write!(f, "'-{val}' is not a valid option")
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
impl<T: fmt::Debug + fmt::Display> std::error::Error for SchmargsError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseInt(err) => Some(err),
            _ => None,
        }
    }
}
