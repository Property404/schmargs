use core::{
    fmt::{self},
    num::ParseIntError,
};
use derive_more::{Display, From};

/// The error type used in this crate
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub enum SchmargsError<T> {
    /// Transparent wrapper around [ParseIntError]
    #[from]
    ParseInt(ParseIntError),
    /// Passed a short flag that doesn't exist
    NoSuchShortFlag(char),
    /// Passed a long flag that doesn't exist
    NoSuchLongFlag(T),
    /// Did not expect this value
    UnexpectedValue(T),
    /// Expected a value to an argument
    ExpectedValue(&'static str),
}

/// A type-stripped version of [SchmargsError], built from [SchmargsError::strip]
#[derive(Clone, Debug, From, Display, PartialEq, Eq)]
pub enum StrippedSchmargsError {
    /// See [SchmargsError::ParseInt]
    #[from]
    ParseInt(ParseIntError),
    /// See [SchmargsError::NoSuchShortFlag]
    #[display("Found invalid option: '-{_0}'")]
    NoSuchShortFlag(char),
    /// See [SchmargsError::NoSuchLongFlag]
    #[display("Found invalid option")]
    NoSuchLongFlag,
    /// See [SchmargsError::UnexpectedValue]
    #[display("Unexpected positional value")]
    UnexpectedValue,
    /// See [SchmargsError::ExpectedValue]
    #[display("Expected value for '{_0}'")]
    ExpectedValue(&'static str),
}

impl<T> SchmargsError<T> {
    /// Strip information from the error type. This is useful if you want use the error outside its
    /// generic's lifetime bounds.
    pub fn strip(self) -> StrippedSchmargsError {
        match self {
            SchmargsError::ParseInt(val) => StrippedSchmargsError::ParseInt(val),
            SchmargsError::NoSuchShortFlag(val) => StrippedSchmargsError::NoSuchShortFlag(val),
            SchmargsError::ExpectedValue(val) => StrippedSchmargsError::ExpectedValue(val),
            SchmargsError::NoSuchLongFlag(_) => StrippedSchmargsError::NoSuchLongFlag,
            SchmargsError::UnexpectedValue(_) => StrippedSchmargsError::UnexpectedValue,
        }
    }
}

impl<T> From<SchmargsError<T>> for StrippedSchmargsError {
    fn from(err: SchmargsError<T>) -> Self {
        err.strip()
    }
}

impl<T: fmt::Display> Display for SchmargsError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ParseInt(err) => err.fmt(f),
            Self::NoSuchShortFlag(val) => {
                write!(f, "{}", StrippedSchmargsError::NoSuchShortFlag(*val))
            }
            Self::NoSuchLongFlag(val) => {
                write!(f, "{}: '{val}'", StrippedSchmargsError::NoSuchLongFlag)
            }
            Self::UnexpectedValue(val) => {
                write!(f, "{}: '{val}'", StrippedSchmargsError::UnexpectedValue)
            }
            Self::ExpectedValue(val) => {
                write!(f, "{}", StrippedSchmargsError::ExpectedValue(val))
            }
        }
    }
}

impl<T: fmt::Debug + fmt::Display> core::error::Error for SchmargsError<T> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ParseInt(err) => Some(err),
            _ => None,
        }
    }
}

impl core::error::Error for StrippedSchmargsError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ParseInt(err) => Some(err),
            _ => None,
        }
    }
}
