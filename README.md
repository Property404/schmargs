# schmargs

A argument parser that can be used with `#[no_std]`

## §Features

* `clap-derive`-inspired derive macro
* `#![no_std]`-friendly
* Optional arguments
* Multi-arg positional arguments and options with [std::vec::Vec](https://doc.rust-lang.org/1.82.0/alloc/vec/struct.Vec.html)
* Custom and default short and long flags
* A wrapper that allows for `--help` functionality

## §Todo

* Improve documentation
* Improve and write tests for help formatting

## §Helper Attributes

### §`schmargs`

This is an optional attribute that should be specified at the top level.

Arguments:

* `name=<str literal>` - The name of the program. Defaults to the crate name.
* `iterates_over=<type>` - The string type that’s being iterated over. This should be the `Item`
  associated type of the [core::iter::Iterator](https://doc.rust-lang.org/1.82.0/core/iter/traits/iterator/trait.Iterator.html) type passed to Schmargs::parse. This defaults
  to `&str` with an appropriate lifetime. If you’re in an `std` environment and plan on parsing
  arguments passed to your program with `Schmargs::parse_env`, `iterates_over` MUST be specified.

### §`args`

This is an optional attribute that should be specified on an argument.

Arguments:

* `short[=<char literal>]` - The short flag of the argument. If no value is provided, it will
  default to the first letter of the argument name.
* `long[=<str literal>]` - The long flag of the argument. If no value is provided, it will
  default to the the argument name.
* `value_name=<str literal>` - Set the value name of the argument. This is only used for the
  help and usage text.
* `default_value[=<expression>]` - Set the default value of the argument. Defaults to
  [Default::default](https://doc.rust-lang.org/1.82.0/core/default/trait.Default.html#tymethod.default)

## §Example

When using in an `std` environment, you generally want to specify `iterates_over` to be
`String`, so you can iterate over [std::env::Args](https://doc.rust-lang.org/1.82.0/std/env/struct.Args.html).

```rust
use schmargs::Schmargs;

/// A program to yell at a cloud
#[derive(Schmargs)]
#[schmargs(iterates_over=String)]
struct Args {
    /// Yell volume, in decibels
    #[arg(short, long)]
    volume: Option<u64>,
    /// Yell length, in nanoseconds
    // This defaults to 1 second
    #[arg(short, long, default_value = 1_000_000_000)]
    length: u64,
    /// Obscenities to yell
    content: Vec<String>,
}

// This parses the arguments passed to the program
let args = Args::parse_env();
println!("{:?}", args.content);
```

## §`#![no_std]` Examples

```rust
use schmargs::Schmargs;

/// A simple memory dump program
#[derive(Schmargs)]
#[schmargs(name = "hexdump")]
struct Args {
    /// Show color
    #[arg(short, long)]
    color: bool,
    /// Disable sanity checks
    #[arg(short = 'f', long = "force")]
    no_null_check: bool,
    /// How many bytes to show per line
    #[arg(short, long)]
    group: Option<u8>, // this is optional
    /// Starting memory address
    start: *const u8, // required positional argument
    /// Number of bytes to read
    len: usize, // required positional argument
}

let args = Args::parse("-f --group 8 0x40000000 256".split_whitespace()).unwrap();
assert_eq!(args.color, false);
assert_eq!(args.no_null_check, true);
assert_eq!(args.group, Some(8));
assert_eq!(args.start, 0x40000000 as *const u8);
assert_eq!(args.len, 256);
```

When strings are involved, you need to add a generic lifetime parameter

```rust
use schmargs::Schmargs;

/// A very important program to greet somebody
#[derive(Schmargs)]
#[schmargs(name = "greet")]
struct Args<'a> {
    /// Should we kick the person's shins after greeting them?
    #[arg(short, long = "kick")]
    kick_shins: bool,
    /// The person to greet
    person: &'a str,
}

let args = Args::parse("Dagan".split_whitespace()).unwrap();
assert_eq!(args.kick_shins, false);
assert_eq!(args.person, "Dagan");
```

## License

MIT OR Apache-2.0
