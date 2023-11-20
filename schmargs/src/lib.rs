#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub trait Schmargs<'a> {
    fn description() -> &'static str;
    fn parse(args: impl Iterator<Item =  &'a str>) -> Self;
}
