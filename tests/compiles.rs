//! This module checks if various forms of `#[extfn]` can be expanded and compiled.

#![allow(unused_mut)]

use extfn::extfn;

#[extfn]
fn trivial(self: bool) -> bool {
    unimplemented!()
}

#[extfn]
/// doc
fn doc(self: bool) -> bool {
    unimplemented!()
}

#[extfn]
#[must_use]
fn attribute(self: bool) -> bool {
    unimplemented!()
}

#[extfn]
fn generic_trait<T: Clone>(self: T) {
    unimplemented!()
}

#[extfn]
fn generic_type<T: Clone>(self: Box<T>) {
    unimplemented!()
}

#[extfn]
fn where_clause<T>(self: Box<T>)
where
    T: Clone,
{
    unimplemented!()
}

#[extfn]
fn impl_trait(self: impl Clone) {
    unimplemented!()
}

#[extfn]
fn mut_self(mut self: bool) {
    unimplemented!()
}

#[extfn]
fn param_pattern(self: bool, (mut _a, mut _b): (bool, bool)) {
    unimplemented!()
}

#[extfn]
fn reference<T>(self: &Option<&T>) {
    unimplemented!()
}

#[extfn]
fn lifetime<'a, 'b, T>(self: &'a Option<&'b T>) {
    unimplemented!()
}

#[extfn]
fn fn_pointer(self: fn(bool) -> bool) {
    unimplemented!()
}

#[extfn]
fn fn_trait(self: impl Fn(bool) -> bool) {
    unimplemented!()
}

#[extfn]
fn dyn_trait(self: &dyn Send) {
    unimplemented!()
}

#[extfn]
fn multi_generic<T, U, V>(self: (T, U, V)) {
    unimplemented!()
}

#[extfn]
fn const_generic<T, const N: usize>(self: [T; N]) {
    unimplemented!()
}

#[extfn]
async fn async_fn(self: bool) {
    unimplemented!()
}

#[extfn]
unsafe fn unsafe_fn(self: bool) {
    unimplemented!()
}

#[extfn]
fn impl_trait_with_nested_ref<'a>(self: impl IntoIterator<Item = &'a String>) {
    unimplemented!()
}
