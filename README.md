# `extfn` - Extension Functions in Rust

[![Latest version](https://img.shields.io/crates/v/extfn.svg)](https://crates.io/crates/extfn)
[![Documentation](https://img.shields.io/docsrs/extfn.svg)](https://docs.rs/extfn)
[![MSRV](https://img.shields.io/crates/msrv/extfn.svg)](https://github.com/ondt/extfn)
[![License](https://img.shields.io/crates/l/extfn.svg)](https://github.com/ondt/extfn)

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

<details>
<summary>Click here to expand macros</summary>

```rust
use extfn::extfn;
use std::cmp::Ordering;
use std::fmt::Display;

trait factorial {
    fn factorial(self) -> u64;
}
impl factorial for u64 {
    fn factorial(self) -> u64 {
        (1..=self).product()
    }
}

trait string_len<_T1> {
    fn string_len(self) -> usize
    where
        _T1: Display;
}
impl<_T1> string_len<_T1> for _T1 {
    fn string_len(self) -> usize
    where
        _T1: Display,
    {
        format!("{self}").len()
    }
}

trait sorted_by<T> {
    fn sorted_by<F>(self, dummy1: F) -> Vec<T>
    where
        F: FnMut(&T, &T) -> Ordering,
        T: Ord;
}
impl<T> sorted_by<T> for Vec<T> {
    fn sorted_by<F>(mut self, compare: F) -> Vec<T>
    where
        F: FnMut(&T, &T) -> Ordering,
        T: Ord,
    {
        self.sort_by(compare);
        self
    }
}

fn main() {
    assert_eq!(6.factorial(), 720);
    assert_eq!(true.string_len(), 4);
    assert_eq!(vec![2, 1, 3].sorted_by(|a, b| b.cmp(a)), vec![3, 2, 1]);
}
```

</details>

## Supported Function Signatures

A list of all supported function signatures can be found in [tests/signatures.rs](tests/signatures.rs). Nearly
everything I could think of is supported, with a few exceptions (see [Fine Print](#fine-print)).

Please report any edge cases where the "extfn transform" (add `#[extfn]` and rename the first parameter to `self`)
doesn't work.

## Implementation Details

The `#[extfn]` macro essentially just converts a function into an extension trait with a single method.

This trait shares it's name with the extension function, allowing us to mark extension functions as `pub` and to import
them just like regular functions using `use example::add1;`, maintaining the illusion:

```rust
mod example {
    use extfn::extfn;
    
    #[extfn]
    pub fn add1(self: usize) -> usize {
        self + 1
    }
}

use example::add1;

fn main() {
    assert_eq!(1.add1(), 2);
}
```

## Prior Art

Extension functions are already implemented in other programming languages:

- [Extension functions in Kotlin](https://kotlinlang.org/docs/extensions.html#extension-functions)
- [Extension methods in C#](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/extension-methods)

As a Rust feature, extension functions have been proposed
[here](https://internals.rust-lang.org/t/idea-simpler-method-syntax-private-helpers/7460),
[here](https://internals.rust-lang.org/t/idea-trait-impl-item-for-ergonomic-extension-traits/12891/4),
[here](https://internals.rust-lang.org/t/postfix-functions/18540),
[here](https://internals.rust-lang.org/t/weird-syntax-idea-s-for-umcs/19200).

## Fine Print

- Const functions are unsupported because of [E0379](https://doc.rust-lang.org/error_codes/E0379.html)
- `self: T::Assoc` is unsupported

