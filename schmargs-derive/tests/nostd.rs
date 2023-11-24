//! Make sure this works in a #![no_std] environment
#![no_std]
use schmargs::Schmargs;

/// Bla bla bla
#[derive(Schmargs)]
struct Args {
    /// First positional argument
    positional: u64,
    /// Second positional argument
    positional2: u64,
    /// Kill all humans?
    #[arg(long)]
    foo: bool,
}

#[test]
fn nostd_basic() {
    let args = Args::parse("--foo 42 255".split_whitespace()).unwrap();
    assert_eq!(args.positional, 42);
    assert_eq!(args.positional2, 255);
    assert_eq!(args.foo, true);
}
