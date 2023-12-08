use crate::error::SchmargsError;

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

impl<U, T: AsRef<str>> SchmargsField<T> for *const U {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        Ok(usize::parse_str(val)? as *const U)
    }
}

impl<U, T: AsRef<str>> SchmargsField<T> for *mut U {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        Ok(usize::parse_str(val)? as *mut U)
    }
}

impl<'a> SchmargsField<&'a str> for &'a str {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<&'a str>> {
        Ok(val)
    }
}

#[cfg(feature = "std")]
impl<'a> SchmargsField<&'a str> for &'a std::path::Path {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<&'a str>> {
        Ok(val.as_ref())
    }
}

#[cfg(feature = "std")]
impl<T: Into<std::path::PathBuf>> SchmargsField<T> for std::path::PathBuf {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        Ok(val.into())
    }
}

#[cfg(feature = "std")]
impl SchmargsField<String> for String {
    fn parse_str(val: String) -> Result<Self, SchmargsError<String>> {
        Ok(val)
    }
}

#[cfg(feature = "std")]
impl<'a> SchmargsField<&'a str> for String {
    fn parse_str(val: &'a str) -> Result<Self, SchmargsError<&'a str>> {
        Ok(val.into())
    }
}

#[cfg(feature = "std")]
impl<T: AsRef<str> + for<'a> From<&'a str>, Item: SchmargsField<T>> SchmargsField<T> for Vec<Item> {
    fn parse_str(val: T) -> Result<Self, SchmargsError<T>> {
        let mut vec = Vec::with_capacity(1);
        for val in val.as_ref().split(',') {
            vec.push(SchmargsField::parse_str(val.into())?);
        }
        Ok(vec)
    }

    fn parse_it(val: T, it: impl Iterator<Item = T>) -> Result<Self, SchmargsError<T>> {
        let mut vec = Vec::with_capacity(1 + it.size_hint().0);
        vec.push(SchmargsField::parse_str(val)?);
        for val in it {
            vec.push(SchmargsField::parse_str(val)?);
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
