use schmargs::Schmargs;
use schmargs_derive::Schmargs;

#[derive(Schmargs)]
struct Args {
    positional: u64,
    positional2: u8,
    foo: bool,
}

fn main() {
    let args = Args::parse("--foo 42 255".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, 42);
    assert_eq!(args.positional2, 255);
}
