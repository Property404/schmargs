#![allow(dead_code)]
use schmargs::{Schmargs, SchmargsField};

#[test]
fn basic() {
    #[derive(Schmargs)]
    /// Bla bla bla
    struct Args {
        /// First positional argument
        positional: u64,
        /// Second positional argument
        positional2: u64,
        /// Kill all humans?
        #[arg(long)]
        foo: bool,
    }

    let args = Args::parse("--foo 42 255".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, 42);
    assert_eq!(args.positional2, 255);
}

#[test]
fn string() {
    #[derive(Schmargs)]
    /// Blee blarg bloo
    struct Args<'a> {
        /// Blarg
        #[arg(long)]
        foo: bool,
        /// Blee
        positional: &'a str,
        /// Bloop
        positional2: &'a str,
        /// Blop
        positional3: u64,
    }

    let args = Args::parse("--foo bar baz 64".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, "bar");
    assert_eq!(args.positional2, "baz");
    assert_eq!(args.positional3, 64);
}

#[test]
fn arbitrary_lifetime_names() {
    #[derive(Schmargs)]
    /// Schmorp
    struct Args<'fuck> {
        /// Schmeerp
        #[arg(long)]
        foo: bool,
        /// Schmripe
        positional: &'fuck str,
    }

    let args = Args::parse("--foo bar".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, "bar");
}

#[test]
fn with_generics() {
    #[derive(Schmargs)]
    /// Flarp
    struct Args<'fuck, T: SchmargsField<&'fuck str>> {
        /// Fleerp
        #[arg(long)]
        foo: bool,
        /// Flurp
        positional: &'fuck str,
        /// Florp
        positional2: T,
    }

    let args = Args::<u64>::parse("--foo well_im_only 82".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, "well_im_only");
    assert_eq!(args.positional2, 82);
}

#[test]
fn unsigned_integers() {
    #[derive(Schmargs)]
    /// Gleep
    struct Args {
        /// Glup
        a: u8,
        /// Glorp
        b: u16,
        /// Glarp
        c: u32,
        /// Glurt
        d: u64,
        /// Glirp
        e: u128,
    }

    let args = Args::parse("0 1 2 0xfe 3141592".split_whitespace()).unwrap();
    assert_eq!(args.a, 0);
    assert_eq!(args.b, 1);
    assert_eq!(args.c, 2);
    assert_eq!(args.d, 0xfe);
    assert_eq!(args.e, 3141592);
}

#[test]
fn signed_integers() {
    #[derive(Schmargs)]
    /// Description goes here
    struct Args {
        /// How many children to kill
        a: i8,
        /// How many toddlers to kill
        b: i16,
        /// How many babies to kill
        c: i32,
        /// How many puppies to kill
        d: i64,
        /// How many kittens to kill
        e: i128,
    }

    let args = Args::parse("0 1 -- -2 0xfe -3141592".split_whitespace()).unwrap();

    assert_eq!(args.a, 0);
    assert_eq!(args.b, 1);
    assert_eq!(args.c, -2);
    assert_eq!(args.d, 0xfe);
    assert_eq!(args.e, -3141592);
}

#[test]
fn pointers() {
    #[derive(Schmargs)]
    /// Description goes here
    struct Args {
        /// Pointer to god knows what
        a: *mut u8,
        /// Pointer to god doesn't knows what
        b: *const (),
    }

    let args = Args::parse("0xfe 0x3141592".split_whitespace()).unwrap();

    assert_eq!(args.a as usize, 0xfe);
    assert_eq!(args.b as usize, 0x3141592);
}

#[test]
fn help_arg() {
    use schmargs::ArgsWithHelp;

    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args {
        /// How many puppies to kick
        puppies: i8,
    }

    let args = ArgsWithHelp::<Args>::parse("--help".split_whitespace()).unwrap();
    assert!(matches!(args, ArgsWithHelp::Help));

    let args = ArgsWithHelp::<Args>::parse("8".split_whitespace()).unwrap();
    assert!(matches!(args, ArgsWithHelp::Args(Args { puppies: 8 })));
}

#[test]
fn short_flags() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args {
        /// Kick adult dogs, too
        #[arg(short = 'a')]
        adults: bool,
        /// How many puppies to kick
        puppies: i8,
    }

    let args = Args::parse("-a 8".split_whitespace()).unwrap();
    assert!(args.adults);
    assert_eq!(args.puppies, 8);
}

#[test]
fn short_flag_default() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args {
        /// Kick adult dogs, too
        #[arg(short)]
        adults: bool,
        /// How many puppies to kick
        puppies: i8,
    }

    let args = Args::parse("-a 8".split_whitespace()).unwrap();
    assert!(args.adults);
    assert_eq!(args.puppies, 8);
}

#[test]
fn specify_custom_long() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args {
        /// Kick adult dogs, too
        #[arg(short, long = "adult")]
        adults: bool,
        /// How many puppies to kick
        puppies: i8,
    }

    let args = Args::parse("--adult 8".split_whitespace()).unwrap();
    assert!(args.adults);
    assert_eq!(args.puppies, 8);
}

#[test]
fn option() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(short, long)]
        puppy: &'a str,
    }

    let args = Args::parse("--puppy eddie".split_whitespace()).unwrap();
    assert_eq!(args.puppy, "eddie");
}

#[test]
fn option_plus_positional() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(short, long)]
        puppy: &'a str,
        /// Number of times to kick
        times_to_kick: u8,
    }

    let args = Args::parse("--puppy eddie 32".split_whitespace()).unwrap();
    assert_eq!(args.puppy, "eddie");
    assert_eq!(args.times_to_kick, 32);
}

#[test]
fn optional_option() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(short, long)]
        puppy: Option<&'a str>,
    }

    let args = Args::parse("".split_whitespace()).unwrap();
    assert_eq!(args.puppy, None);
}

#[test]
fn name_and_description() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(short, long)]
        _puppy: Option<&'a str>,
    }

    assert_eq!(Args::NAME, "pupkick");
    assert_eq!(Args::DESCRIPTION, "Automatic puppy kicker");
}

#[test]
fn usage_text() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        puppy: Option<&'a str>,
    }

    assert_eq!(Args::USAGE, "pupkick [PUPPY]");
}

#[test]
fn custom_value_name() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(value_name = "KITTEN")]
        puppy: Option<&'a str>,
    }

    assert_eq!(Args::USAGE, "pupkick [KITTEN]");
}

#[test]
fn help_text() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        puppy: Option<&'a str>,
    }

    assert_eq!(
        format!("{}", Args::help()),
        "Automatic puppy kicker

Usage: pupkick [PUPPY]

Arguments:
[PUPPY]        The puppy to kick"
    );
}

#[test]
fn help_text2() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// Eat the puppy after kicking it?
        #[arg(short, long)]
        eat: bool,
        /// The puppy to kick
        puppy: &'a str,
    }

    assert_eq!(
        format!("{}", Args::help()),
        "Automatic puppy kicker

Usage: pupkick [OPTIONS] PUPPY

Arguments:
PUPPY        The puppy to kick

Options:
-e, --eat Eat the puppy after kicking it?"
    );
}

#[test]
fn translate_underscore_to_hyphens() {
    #[derive(Schmargs)]
    #[schmargs(name = "pupkick")]
    /// Automatic puppy kicker
    struct Args<'a> {
        /// The puppy to kick
        #[arg(short, long)]
        puppy_to_kick: Option<&'a str>,
    }

    Args::parse("--puppy-to-kick joe".split_whitespace()).unwrap();
}

#[cfg(feature = "std")]
mod with_feature_std {
    use super::*;

    #[test]
    fn owned_string() {
        #[derive(Schmargs)]
        #[schmargs(iterates_over = String)]
        /// Automatic puppy kicker
        struct Args {
            /// The puppy to kick
            puppy: String,
        }

        let arguments = vec![String::from("Gus")];

        let args = Args::parse(arguments.into_iter()).unwrap();
        assert_eq!(args.puppy, String::from("Gus"));
    }

    #[test]
    fn nonpositional_string_vec_by_commas() {
        #[derive(Schmargs)]
        #[schmargs(iterates_over = String)]
        /// Automatic puppy kicker
        struct Args {
            /// Which puppies to kick
            #[arg(short, long)]
            puppies: Vec<String>,
            /// Numbers to shout while kicking puppies
            numbers: Vec<usize>,
        }

        let arguments = "--puppies Billy,Samantha,Muffin 3 1 4 1 5"
            .split_whitespace()
            .map(ToString::to_string);

        let args = Args::parse(arguments).unwrap();
        assert_eq!(args.puppies, vec!["Billy", "Samantha", "Muffin"]);
        assert_eq!(args.numbers, vec![3, 1, 4, 1, 5]);
    }

    #[test]
    fn positional_string_vec() {
        #[derive(Schmargs)]
        #[schmargs(iterates_over = String)]
        /// Automatic puppy kicker
        struct Args {
            /// Which puppies to kick
            puppies: Vec<String>,
        }

        let arguments = "Billy Samantha Muffin"
            .split_whitespace()
            .map(ToString::to_string);

        let args = Args::parse(arguments).unwrap();
        assert_eq!(args.puppies, vec!["Billy", "Samantha", "Muffin"]);
    }

    #[test]
    fn path() {
        use std::path::PathBuf;

        #[derive(Schmargs)]
        #[schmargs(iterates_over = String)]
        /// Automatic puppy kicker
        struct Args {
            /// Path in which puppies file is located
            puppy_file: PathBuf,
        }

        let arguments = "/path/to/file".split_whitespace().map(ToString::to_string);

        let args = Args::parse(arguments).unwrap();
        assert_eq!(args.puppy_file, PathBuf::from("/path/to/file"));
    }
}
