use schmargs::{Schmargs, SchmargsField};
use schmargs_derive::Schmargs;

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
    struct Args<'fuck, T: SchmargsField<'fuck>> {
        /// Fleerp
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
        puppies: i8,
    }

    assert_eq!(Args::description(), "Automatic puppy kicker");
}
