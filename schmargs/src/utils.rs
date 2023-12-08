//! Parsing utilities for internal use
#[derive(Debug, PartialEq, Eq)]
#[doc(hidden)]
pub enum DumbArgument<T> {
    ShortFlags(T),
    LongFlag(T),
    Positional(T),
}

impl<T> DumbArgument<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::ShortFlags(inner) | Self::LongFlag(inner) | Self::Positional(inner) => inner,
        }
    }
}

/// An iterator that parses out short flags (`-s`), long flags(`--long`), and values out of an
/// iterator of arguments
#[doc(hidden)]
pub struct DumbIterator<T: AsRef<str>, InputIterator: Iterator<Item = T>> {
    hit_double_dash: bool,
    args: InputIterator,
}

impl<T: AsRef<str>, InputIterator: Iterator<Item = T>> DumbIterator<T, InputIterator> {
    /// Construct from list of logical arguments
    pub fn from_args(args: InputIterator) -> Self {
        Self {
            hit_double_dash: false,
            args,
        }
    }
}

impl<T: AsRef<str>, InputIterator: Iterator<Item = T>> Iterator for DumbIterator<T, InputIterator> {
    type Item = DumbArgument<T>;

    fn next(&mut self) -> Option<DumbArgument<T>> {
        let Some(arg) = self.args.next() else {
            return None;
        };

        if self.hit_double_dash {
            return Some(DumbArgument::Positional(arg));
        }

        if let Some(stripped_arg) = arg.as_ref().strip_prefix("--") {
            if stripped_arg.is_empty() {
                self.hit_double_dash = true;
                return self.next();
            }
            Some(DumbArgument::LongFlag(arg))
        } else if arg.as_ref().starts_with('-') {
            Some(DumbArgument::ShortFlags(arg))
        } else {
            Some(DumbArgument::Positional(arg))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = self.args.size_hint();
        (sh.0, sh.1)
    }
}
