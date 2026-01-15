#![allow(dead_code)]

#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned)]
struct Simplest<'a> {
    field: Cow<'a, str>,
}

#[derive(IntoOwned)]
struct PaddedWithDifferent<'a, 'b> {
    a: bool,
    b: u32,
    c: Cow<'a, str>,
    d: Simplest<'b>,
}

#[derive(IntoOwned)]
struct PaddedWithSame<'a> {
    a: bool,
    b: u32,
    c: Cow<'a, str>,
    d: Simplest<'a>,
}

#[test]
fn simplest() {
    let non_static_string = "foobar".to_string();

    let simplest = Simplest {
        field: Cow::Borrowed(&non_static_string),
    };

    accepts_only_static(simplest.into_owned());
}

fn accepts_only_static(s: Simplest<'static>) {
    drop(s)
}
