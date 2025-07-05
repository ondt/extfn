# `extfn` - Extension Functions in Rust

`extfn` is a Rust library that implements _extension functions_, allowing any[*](#fine-print) freestanding function to
be called as `a.foo(b)` instead of `foo(a, b)` just by adding `#[extfn]` and renaming the first parameter to `self`.

```rust
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
```

A list of all supported function forms can be found [here](tests/compiles.rs).

## Prior Art

Extension functions are already implemented in [Kotlin](https://kotlinlang.org/docs/extensions.html#extension-functions)
and [C#](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/extension-methods).

As a Rust feature, extension functions have been proposed
[here](https://internals.rust-lang.org/t/idea-simpler-method-syntax-private-helpers/7460),
[here](https://internals.rust-lang.org/t/idea-trait-impl-item-for-ergonomic-extension-traits/12891/4),
[here](https://internals.rust-lang.org/t/postfix-functions/18540), and
[here](https://internals.rust-lang.org/t/weird-syntax-idea-s-for-umcs/19200).

## Fine Print

- Const functions are unsupported because of [E0379](https://doc.rust-lang.org/error_codes/E0379.html)
- `self: T::Assoc` is unsupported

