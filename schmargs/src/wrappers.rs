//! Schmargs wrappers that provide additional functionality, like `--version` and `--help` options
use crate::{Argument, Schmargs, SchmargsError};
use core::fmt;

pub enum ArgsWithHelp<T: for<'a> Schmargs<'a>> {
    Help,
    Args(T),
}

impl<'a, T: for<'b> Schmargs<'b>> Schmargs<'a> for ArgsWithHelp<T> {
    fn description() -> &'static str {
        T::description()
    }

    fn write_help_with_min_indent(
        mut f: impl fmt::Write,
        name: impl AsRef<str>,
        min_indent: usize,
    ) -> Result<usize, fmt::Error> {
        let prefix = "-h, --help";
        let min_indent = core::cmp::max(min_indent, prefix.len() + 1);
        let min_indent = core::cmp::max(
            min_indent,
            T::write_help_with_min_indent(&mut f, name, min_indent)?,
        );
        writeln!(f)?;
        write!(f, "{}", prefix)?;
        for _ in 0..(min_indent - prefix.len()) {
            write!(f, " ")?;
        }
        write!(f, "Print help")?;
        Ok(min_indent)
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
