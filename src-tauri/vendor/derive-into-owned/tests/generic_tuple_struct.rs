#[macro_use]
extern crate derive_into_owned;

use std::borrow::Cow;
use std::borrow::ToOwned;

#[derive(IntoOwned)]
struct Foo<'a, T: 'a + ToOwned + ?Sized>(Cow<'a, T>);

#[test]
fn tuple_struct() {
    let non_static_string: String = "foobar".to_string();

    let thing = Foo(Cow::Borrowed(&non_static_string));

    accepts_only_static(thing.into_owned());

    let non_static_vec: Vec<u8> = vec![0u8; 8];

    let thing = Foo(Cow::Borrowed(&non_static_vec[..]));

    accepts_only_static(thing.into_owned());
}

fn accepts_only_static<T: ToOwned + 'static + ?Sized>(static_foo: Foo<'static, T>) {
    drop(static_foo);
}
