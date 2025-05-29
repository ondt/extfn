use extfn::extfn;
use std::cmp::Ordering;
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
fn sorted_by<T: Ord>(mut self: Vec<T>, f: impl FnMut(&T, &T) -> Ordering) -> Vec<T> {
    self.sort_by(f);
    self
}

fn main() {
    assert_eq!(1.add1(), 2);
    assert_eq!(true.string_len(), 4);
    assert_eq!(vec![2, 1, 3].sorted_by(|a, b| b.cmp(a)), vec![3, 2, 1]);
}
