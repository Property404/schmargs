use schmargs::Schmargs;
use schmargs_derive::Schmargs;

#[derive(Schmargs)]
struct Args {
    foo: bool,
    positional: u64,
}

fn main() {
    let args = Args::parse("--foo 42 fear".split_whitespace()).unwrap();
    assert!(args.foo);
    assert_eq!(args.positional, 42);
}
