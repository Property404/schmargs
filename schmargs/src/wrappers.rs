// Schmargs wrappers that provide additional functionality, like `--version` and `--help` options
use crate::{Schmargs, SchmargsError};
use core::fmt;

/// A generic wrapper
pub trait Wrapper: Sized {
    const SHORT_OPTION: char;
    const LONG_OPTION: &'static str;
    const DESCRIPTION: &'static str;
    type SchmargsType;
    fn parsed(inner: Self::SchmargsType) -> Self;
    fn special() -> Self;
}

impl<'a, W: Wrapper> Schmargs<'a> for W
where
    W::SchmargsType: Schmargs<'a>,
    <W::SchmargsType as Schmargs<'a>>::Item: AsRef<str>,
{
    type Item = <W::SchmargsType as Schmargs<'a>>::Item;

    fn name() -> &'static str {
        W::SchmargsType::name()
    }

    fn version() -> &'static str {
        W::SchmargsType::version()
    }

    fn description() -> &'static str {
        W::SchmargsType::description()
    }

    fn write_help_with_min_indent(
        mut f: impl fmt::Write,
        min_indent: usize,
    ) -> Result<usize, fmt::Error> {
        let prefix_len = "-h, ".len() + W::LONG_OPTION.len();
        let min_indent = core::cmp::max(min_indent, prefix_len + 1);
        let min_indent = core::cmp::max(
            min_indent,
            W::SchmargsType::write_help_with_min_indent(&mut f, min_indent)?,
        );
        writeln!(f)?;
        write!(f, "-{}, {}", W::SHORT_OPTION, W::LONG_OPTION)?;
        for _ in 0..(min_indent - prefix_len) {
            write!(f, " ")?;
        }
        write!(f, "{}", W::DESCRIPTION)?;
        Ok(min_indent)
    }

    fn parse(args: impl Iterator<Item = Self::Item>) -> Result<Self, SchmargsError<Self::Item>> {
        match W::SchmargsType::parse(args) {
            Ok(inner) => Ok(W::parsed(inner)),
            Err(inner) => {
                match inner {
                    SchmargsError::NoSuchShortFlag(val) => {
                        if val == W::SHORT_OPTION {
                            return Ok(W::special());
                        }
                    }
                    SchmargsError::NoSuchLongFlag(val) if val.as_ref() == W::LONG_OPTION => {
                        return Ok(W::special());
                    }
                    _ => {}
                }
                Err(inner)
            }
        }
    }
}

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
/// let args = Args::parse("--help".split_whitespace()).unwrap();
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

impl<S> Wrapper for ArgsWithHelp<S> {
    const SHORT_OPTION: char = 'h';
    const LONG_OPTION: &'static str = "--help";
    const DESCRIPTION: &'static str = "Print help";
    type SchmargsType = S;

    fn parsed(inner: Self::SchmargsType) -> Self {
        Self::Args(inner)
    }

    fn special() -> Self {
        Self::Help
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

/// A wrapper that provides `--version` functionality
///
/// This can be used with [ArgsWithHelp] or by itself
///
/// # Example
/// ```
/// use schmargs::{ArgsWithHelp, ArgsWithVersion, Schmargs};
///
/// /// Program that barks
/// #[derive(Schmargs)]
/// #[schmargs(name = "greet")]
/// struct BareArgs {
///     /// Should we meow instead?
///     #[arg(short, long)]
///     meow: bool,
/// }
/// type Args = ArgsWithHelp<ArgsWithVersion<BareArgs>>;
///
/// let args = Args::parse("--version".split_whitespace()).unwrap();
/// match args {
///     Args::Args(ArgsWithVersion::Args(args)) => {
///         if args.meow {
///             println!("Meow!");
///         } else {
///             println!("Bark!");
///         }
///     }
///     // Print the version
///     Args::Args(ArgsWithVersion::Version) => {
///         println!("{}", Args::version());
///     }
///     // Print help
///     Args::Help => {
///         println!("{}", Args::Help);
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ArgsWithVersion<S> {
    /// User passed the '-v' or '--version' flag
    Version,
    /// Parsed arguments
    Args(S),
}

impl<S> Wrapper for ArgsWithVersion<S> {
    const SHORT_OPTION: char = 'v';
    const LONG_OPTION: &'static str = "--version";
    const DESCRIPTION: &'static str = "Print version";
    type SchmargsType = S;

    fn parsed(inner: Self::SchmargsType) -> Self {
        Self::Args(inner)
    }

    fn special() -> Self {
        Self::Version
    }
}

impl<'a, S: Schmargs<'a>> fmt::Display for ArgsWithVersion<S>
where
    <S as Schmargs<'a>>::Item: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::write_help_with_min_indent(f, 0)?;
        Ok(())
    }
}
