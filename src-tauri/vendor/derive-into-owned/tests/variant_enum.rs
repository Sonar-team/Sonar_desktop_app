#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned)]
enum Foo<'a> {
    Str { a: Cow<'a, str>, b: u32 },
    Bytes { a: bool, b: Cow<'a, [u8]> },
}

#[test]
fn enum_with_only_cow_variants() {
    let s = "foobar".to_string();
    let v = b"12345234".to_vec();

    let thing = Foo::Str {
        a: Cow::Borrowed(&s),
        b: 32,
    };
    accepts_only_static(thing.into_owned());

    let thing = Foo::Bytes {
        a: false,
        b: Cow::Borrowed(&v[..]),
    };
    accepts_only_static(thing.into_owned());
}

fn accepts_only_static<T: 'static>(anything: T) {
    drop(anything)
}
