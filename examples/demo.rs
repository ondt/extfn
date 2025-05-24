use extfn::extfn;
use std::fmt::Display;

#[extfn]
fn add1(self: usize) -> usize {
    self + 1
}

#[extfn]
fn string_len(self: impl Display) -> usize {
    format!("{self}").len()
}

fn main() {
    assert_eq!(1.add1(), 2);
    assert_eq!(true.string_len(), 4);
}
