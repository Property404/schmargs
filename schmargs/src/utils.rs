//! Parsing utilities, mostly for internal use
use crate::Argument;

/// An iterator that parses out short flags (`-s`), long flags(`--long`), and values out of an
/// iterator of arguments
pub struct ArgumentIterator<'a, I: Iterator<Item = &'a str>> {
    hit_double_dash: bool,
    shortflags: Option<core::str::Chars<'a>>,
    args: I,
}

impl<'a, I: Iterator<Item = &'a str>> ArgumentIterator<'a, I> {
    /// Construct from list of logical arguments
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
