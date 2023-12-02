//! Schmargs wrappers that provide additional functionality, like `--version` and `--help` options
use crate::{Argument, Schmargs, SchmargsError};
use core::fmt;

/// A wrapper that provides `--help` functionality
///
/// # Example
///
/// Without lifetimes
/// ```
/// use schmargs::{Schmargs, wrappers::ArgsWithHelp};
///
/// /// Program that barks
/// #[derive(Schmargs)]
/// struct BareArgs {
///     /// Should we meow instead?
///     #[arg(short,long)]
///     meow: bool,
/// }
/// type Args = ArgsWithHelp::<BareArgs>;
///
/// let args = ArgsWithHelp::parse("--help".split_whitespace()).unwrap();
/// match args {
///     Args::Args(args) => {
///         if args.meow {
///             println!("Meow!");
///         } else {
///             println!("Bark!");
///         }
///     },
///     Args::Help => {
///         let mut s = String::new();
///         Args::write_help(&mut s, "greet").unwrap();
///         println!("{s}");
///     }
/// }
/// ```
///
/// With lifetimes
/// ```
/// use schmargs::{Schmargs, wrappers::ArgsWithHelp};
///
/// /// A very important program to greet somebody
/// #[derive(Schmargs)]
/// struct BareArgs<'a> {
///     /// Should we kick the person's shins after greeting them?
///     #[arg(short,long="kick")]
///     kick_shins: bool,
///     /// Name of the person we're greeting
///     person: &'a str
/// }
/// type Args<'a> = ArgsWithHelp::<BareArgs<'a>>;
///
/// let args = ArgsWithHelp::parse("--help".split_whitespace()).unwrap();
/// match args {
///     Args::Args(args) => {
///         println!("Hello, {}!", args.person);
///         if args.kick_shins {
///             println!("Now I'm gonna kick your shins!");
///         }
///     },
///     Args::Help => {
///         let mut s = String::new();
///         Args::write_help(&mut s, "greet").unwrap();
///         println!("{s}");
///     }
/// }
/// ```
pub enum ArgsWithHelp<S: Sized> {
    Help,
    Args(S),
}

impl<T: AsRef<str>, S: Schmargs<T>> Schmargs<T> for ArgsWithHelp<S> {
    fn description() -> &'static str {
        S::description()
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
            S::write_help_with_min_indent(&mut f, name, min_indent)?,
        );
        writeln!(f)?;
        write!(f, "{}", prefix)?;
        for _ in 0..(min_indent - prefix.len()) {
            write!(f, " ")?;
        }
        write!(f, "Print help")?;
        Ok(min_indent)
    }

    fn parse(args: impl Iterator<Item = T>) -> Result<Self, SchmargsError<T>> {
        match S::parse(args) {
            Ok(inner) => Ok(Self::Args(inner)),
            Err(inner) => {
                if let SchmargsError::NoSuchOption(ref option) = inner {
                    match option {
                        Argument::LongFlag(v) if v.as_ref() == "--help" => {
                            return Ok(Self::Help);
                        }
                        Argument::ShortFlag('h') => {
                            return Ok(Self::Help);
                        }
                        _ => {}
                    }
                }
                Err(inner)
            }
        }
    }
}
