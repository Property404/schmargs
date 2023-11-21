use schmargs::Schmargs;
use schmargs_derive::Schmargs;

#[test]
fn basic() {
    #[derive(Schmargs)]
    struct Args {
        positional: u64,
        positional2: u8,
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
    struct Args<'a> {
        foo: bool,
        positional: &'a str,
        positional2: &'a str,
        positional3: u64
    }

    let args = Args::parse("--foo bar baz 64".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, "bar");
    assert_eq!(args.positional2, "baz");
    assert_eq!(args.positional3, 64);
}
