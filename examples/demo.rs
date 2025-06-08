use extfn::extfn;
use std::cmp::Ordering;
use std::fmt::Display;

#[extfn]
fn factorial(self: u64) -> u64 {
    (1..=self).product()
}

#[extfn]
fn string_len(self: impl Display) -> usize {
    format!("{self}").len()
}

#[extfn]
fn sorted_by<T: Ord, F>(mut self: Vec<T>, compare: F) -> Vec<T>
where
    F: FnMut(&T, &T) -> Ordering,
{
    self.sort_by(compare);
    self
}

fn main() {
    assert_eq!(6.factorial(), 720);
    assert_eq!(true.string_len(), 4);
    assert_eq!(vec![2, 1, 3].sorted_by(|a, b| b.cmp(a)), vec![3, 2, 1]);
}
