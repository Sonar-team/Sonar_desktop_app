#![allow(dead_code)]

#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned, Borrowed)]
struct Thing<'a> {
    bees: Vec<Bar<'a>>,
    cees: Vec<Cow<'a, str>>,
}

#[derive(IntoOwned, Borrowed)]
struct Bar<'a> {
    s: Cow<'a, str>,
}

#[test]
fn vec() {
    let local = "asdf".to_string();
    let thing = Thing {
        bees: vec![Bar {
            s: Cow::Borrowed(&local),
        }],
        cees: vec![],
    };
    accept_static(thing.into_owned());
}

fn accept_static(thing: Thing<'static>) {
    drop(thing);
}
