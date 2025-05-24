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

#[extfn]
fn sorted<T: Ord>(mut self: Vec<T>) -> Self {
    self.sort();
    self
}

fn main() {
    assert_eq!(1.add1(), 2);
    assert_eq!(true.string_len(), 4);
    assert_eq!(vec![3, 1, 2].sorted(), vec![1, 2, 3]);
}
