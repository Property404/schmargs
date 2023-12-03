# schmargs

A `#![no_std]` argument parser

## Example

```rust
use schmargs::Schmargs;

/// A simple memory dump program
#[derive(Schmargs)]
struct Args {
    /// Show color
    #[arg(short, long)]
    color: bool,
    /// Disable sanity checks
    #[arg(short='f', long="force")]
    no_null_check: bool,
    /// How many bytes to show per line
    #[arg(short, long)]
    group: Option<u8>, // this is optional
    /// Starting memory address
    start: usize, // required positional argument
    /// Number of bytes to read
    len: usize // required positional argument
}

let args = Args::parse("-f --group 8 0x40000000 256".split_whitespace()).unwrap();
assert_eq!(args.color, false);
assert_eq!(args.no_null_check, true);
assert_eq!(args.group, Some(8));
assert_eq!(args.start, 0x40000000);
assert_eq!(args.len, 256);

```

When strings are involved, you need to add a generic lifetime parameter

```rust
use schmargs::Schmargs;

/// A very important program to greet somebody
#[derive(Schmargs)]
struct Args<'a> {
    /// Should we kick the person's shins after greeting them?
    #[arg(short,long="kick")]
    kick_shins: bool,
    /// The person to greet
    person: &'a str
}

let args = Args::parse("Dagan".split_whitespace()).unwrap();
assert_eq!(args.kick_shins, false);
assert_eq!(args.person, "Dagan");
```
