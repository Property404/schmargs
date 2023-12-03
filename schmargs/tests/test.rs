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
fn description() {
    #[derive(Schmargs)]
    /// Automatic puppy kicker
    struct Args {
        /// How many puppies to kick
        _puppies: i8,
    }

    assert_eq!(Args::description(), "Automatic puppy kicker");
}

#[test]
fn help_arg() {
    use schmargs::wrappers::ArgsWithHelp;

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
}
