#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned, Borrowed)]
struct Foo<'a> {
    a: Cow<'a, str>,
    b: Option<Bar<'a>>,
}

#[derive(IntoOwned, Borrowed)]
struct Bar<'a> {
    c: Cow<'a, [u8]>,
    s: SomeCloneType,
}

#[derive(Clone)]
struct SomeCloneType {
    #[allow(dead_code)]
    foo: u32,
}

#[test]
fn borrowed() {
    let owned = Foo {
        a: Cow::Borrowed("str"),
        b: None,
    }
    .into_owned();

    let borrowed = owned.borrowed();

    // owned cannot be moved while borrowed exists
    test(&owned, borrowed);
}

fn test<'b, 'a: 'b>(lives_longer: &Foo<'a>, lives_less: Foo<'b>) {
    drop(lives_less);
    #[allow(clippy::drop_ref)]
    drop(lives_longer);
}
