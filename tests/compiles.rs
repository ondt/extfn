//! This module checks if various forms of `#[extfn]` can be expanded and compiled.

#![allow(unused_mut)]
#![allow(clippy::needless_lifetimes)]
#![deny(warnings)]

use extfn::extfn;

#[extfn]
fn trivial(self: bool) -> bool {
    unimplemented!()
}

#[extfn]
pub fn visibility(self: bool) {
    unimplemented!()
}

#[extfn]
/// doc
fn doc(self: bool) -> bool {
    unimplemented!()
}

#[extfn]
#[inline]
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
fn const_generic<T: Sync, const N: usize>(self: [T; N]) {
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

#[extfn]
fn complex_impl_trait_ref(self: &mut impl Ord) -> Self {
    unimplemented!()
}

#[extfn]
fn complex_impl_trait_multi(self: Result<impl Ord, impl Eq>) -> Self {
    unimplemented!()
}

#[extfn]
fn second_generic_lifetime<'a, T: Ord>(self: T, _second: &'a str) {
    unimplemented!()
}

#[extfn]
fn second_generic_const<T: Ord, const N: usize>(self: T, _second: [(); N]) {
    unimplemented!()
}

#[extfn]
fn second_generic_type<T: Ord, U: Eq>(self: T, _second: U) {
    unimplemented!()
}

#[extfn]
fn complex_where_clause_with_second_generic<'s, T, U>(self: Box<&'s T>, _second: &'s U)
where
    T: Clone,
    T: 's,
    Result<&'s T, &'s U>: Clone,
    String: AsRef<&'s T>,
    for<'a> &'a str: AsRef<U>,
{
    unimplemented!()
}

#[extfn]
fn same_generic_twice<T>(self: T, _second: T) {
    unimplemented!()
}

#[extfn]
fn generic_under_impl_trait<E: Eq>(self: impl From<E>) {
    unimplemented!()
}

#[extfn]
fn lifetime_unelided<'a, T>(self: &'a T) -> &'a T {
    unimplemented!()
}

#[extfn]
fn lifetime_elided<T>(self: &T) -> &T {
    unimplemented!()
}

#[extfn]
fn lifetime_elided_multi<T>(self: &T, _second: &T) -> &T {
    unimplemented!()
}

#[extfn]
fn extra_paren_1<T>(#[allow(warnings)] self: (((((&T)))))) -> &T {
    unimplemented!()
}

#[extfn]
fn extra_paren_2(self: &(impl Sync + 'static)) -> &str {
    unimplemented!()
}

#[extfn]
fn nested_generics<'a, E: Eq + 'a, F: for<'f> From<[&'f &'a E; N]>, const N: usize>(
    self: F,
    _second: E,
) {
    unimplemented!()
}
