#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;

#[derive(IntoOwned)]
enum Foo<'a> {
    Str(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
}

#[test]
fn enum_with_only_cow_variants() {
    let s = "foobar".to_string();
    let v = b"12345234".to_vec();

    let thing = Foo::Str(Cow::Borrowed(&s));
    accepts_only_static(thing.into_owned());

    let thing = Foo::Bytes(Cow::Borrowed(&v[..]));
    accepts_only_static(thing.into_owned());
}

fn accepts_only_static<T: 'static>(anything: T) {
    drop(anything)
}
