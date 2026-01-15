#![allow(dead_code)]

#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned, Clone, PartialEq, Debug)]
struct Foo<'a> {
    field: Option<Cow<'a, str>>,
}

#[derive(IntoOwned)]
struct Wild<'a> {
    field: Option<Option<Option<Option<Cow<'a, str>>>>>,
}

#[derive(IntoOwned)]
struct Wilder<'a> {
    field: Option<Wild<'a>>,
}

#[test]
fn opt_cow_field() {
    let s = "foobar".to_string();

    let thing = Foo {
        field: Some(Cow::Borrowed(&s)),
    };
    assert_eq!(thing.clone().into_owned(), thing);
    accepts_only_static(thing.into_owned());

    let thing = Foo { field: None };
    accepts_only_static(thing.into_owned());
}

fn accepts_only_static<T: 'static>(anything: T) {
    drop(anything)
}
