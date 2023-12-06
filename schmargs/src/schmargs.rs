use crate::SchmargsError;
use core::fmt;

/// An argument parser
pub trait Schmargs<'a>: Sized {
    /// The item [Schmargs::parse] will iterate over. This is typically &str or [String]
    type Item;

    /// Name of the command
    const NAME: &'static str;

    /// Command version
    const VERSION: &'static str;

    /// Command description
    const DESCRIPTION: &'static str;

    /// Write help text to `f`
    /// Returns the indent used, which will be greater than or equal to `min_indent`
    ///
    /// Unless you're implementing [Schmargs], you most likely want to use the
    /// [Display](core::fmt::Display) impl:
    ///
    /// ```
    /// use schmargs::Schmargs;
    ///
    /// /// Fake program
    /// #[derive(Schmargs)]
    /// struct Args {}
    ///
    /// let args = Args::parse("".split_whitespace()).unwrap();
    ///
    /// println!("{args}");
    /// ```
    fn write_help_with_min_indent(
        f: impl fmt::Write,
        min_indent: usize,
    ) -> Result<usize, fmt::Error>;

    /// Construct from an iterator of arguments
    fn parse(args: impl Iterator<Item = Self::Item>) -> Result<Self, SchmargsError<Self::Item>>;

    /// Convenience function to parse from [std::env::args]
    ///
    /// Note that this will exit the program on error. If this is not the behavior you want, use
    /// [Schmargs::parse]
    ///
    /// Must be used with `#[schmargs(iterates_over=String)]`
    #[cfg(feature = "std")]
    fn parse_env() -> Self
    where
        Self::Item: From<String> + fmt::Display,
    {
        let args = std::env::args().skip(1).map(Into::into);

        match Self::parse(args) {
            Ok(args) => args,
            Err(err) => {
                eprintln!("{}: error: {err}", Self::NAME);
                std::process::exit(1);
            }
        }
    }
}
