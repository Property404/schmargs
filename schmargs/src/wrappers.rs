//! Schmargs wrappers that provide additional functionality, like `--version` and `--help` options
use crate::{Schmargs, SchmargsError};
use core::fmt;

/// A wrapper that provides `--help` functionality
///
/// # Example
///
/// Without lifetimes
/// ```
/// use schmargs::{ArgsWithHelp, Schmargs};
///
/// /// Program that barks
/// #[derive(Schmargs)]
/// #[schmargs(name = "greet")]
/// struct BareArgs {
///     /// Should we meow instead?
///     #[arg(short, long)]
///     meow: bool,
/// }
/// type Args = ArgsWithHelp<BareArgs>;
///
/// let args = ArgsWithHelp::parse("--help".split_whitespace()).unwrap();
/// match args {
///     Args::Args(args) => {
///         if args.meow {
///             println!("Meow!");
///         } else {
///             println!("Bark!");
///         }
///     }
///     Args::Help => {
///         println!("{}", Args::Help);
///     }
/// }
/// ```
///
/// With lifetimes
/// ```
/// use schmargs::{ArgsWithHelp, Schmargs};
///
/// /// A very important program to greet somebody
/// #[derive(Schmargs)]
/// #[schmargs(name = "greet")]
/// struct BareArgs<'a> {
///     /// Should we kick the person's shins after greeting them?
///     #[arg(short, long = "kick")]
///     kick_shins: bool,
///     /// Name of the person we're greeting
///     person: &'a str,
/// }
/// type Args<'a> = ArgsWithHelp<BareArgs<'a>>;
///
/// let args = ArgsWithHelp::parse("--help".split_whitespace()).unwrap();
/// match args {
///     Args::Args(args) => {
///         println!("Hello, {}!", args.person);
///         if args.kick_shins {
///             println!("Now I'm gonna kick your shins!");
///         }
///     }
///     Args::Help => {
///         println!("{}", Args::Help);
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ArgsWithHelp<S> {
    /// User passed the '-h' or '--help' flag
    Help,
    /// Parsed arguments
    Args(S),
}

impl<'a, S: Schmargs<'a>> Schmargs<'a> for ArgsWithHelp<S>
where
    S::Item: AsRef<str>,
{
    type Item = S::Item;

    fn name() -> &'static str {
        S::name()
    }

    fn description() -> &'static str {
        S::description()
    }

    fn write_help_with_min_indent(
        mut f: impl fmt::Write,
        min_indent: usize,
    ) -> Result<usize, fmt::Error> {
        let prefix = "-h, --help";
        let min_indent = core::cmp::max(min_indent, prefix.len() + 1);
        let min_indent = core::cmp::max(
            min_indent,
            S::write_help_with_min_indent(&mut f, min_indent)?,
        );
        writeln!(f)?;
        write!(f, "{}", prefix)?;
        for _ in 0..(min_indent - prefix.len()) {
            write!(f, " ")?;
        }
        write!(f, "Print help")?;
        Ok(min_indent)
    }

    fn parse(args: impl Iterator<Item = Self::Item>) -> Result<Self, SchmargsError<Self::Item>> {
        match S::parse(args) {
            Ok(inner) => Ok(Self::Args(inner)),
            Err(inner) => {
                match inner {
                    SchmargsError::NoSuchShortFlag('h') => {
                        return Ok(Self::Help);
                    }
                    SchmargsError::NoSuchLongFlag(val) if val.as_ref() == "--help" => {
                        return Ok(Self::Help);
                    }
                    _ => {}
                }
                Err(inner)
            }
        }
    }
}

impl<'a, S: Schmargs<'a>> fmt::Display for ArgsWithHelp<S>
where
    <S as Schmargs<'a>>::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::write_help_with_min_indent(f, 0)?;
        Ok(())
    }
}
