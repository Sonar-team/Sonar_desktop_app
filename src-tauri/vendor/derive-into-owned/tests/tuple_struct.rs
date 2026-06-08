#![allow(dead_code)]

#[macro_use]
extern crate derive_into_owned;

use std::borrow;
use std::borrow::Cow;

#[derive(IntoOwned)]
struct Foo<'a>(Cow<'a, str>);

#[derive(IntoOwned)]
struct FooExtraFields<'a>(u32, Cow<'a, str>, bool, Vec<bool>);

#[derive(IntoOwned)]
struct Bar<'a>(::std::borrow::Cow<'a, str>);

#[derive(IntoOwned)]
struct Car<'a>(std::borrow::Cow<'a, str>);

#[derive(IntoOwned)]
struct Dar<'a>(borrow::Cow<'a, str>);

#[test]
fn tuple_struct() {
    let non_static_string: String = "foobar".to_string();

    let thing = Foo(Cow::Borrowed(&non_static_string));

    accepts_only_static(thing.into_owned());
}

fn accepts_only_static(static_foo: Foo<'static>) {
    drop(static_foo);
}
