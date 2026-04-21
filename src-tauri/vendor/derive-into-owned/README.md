# derive-into-owned

[![Build Status](https://travis-ci.org/koivunej/derive-into-owned.svg?branch=master)](https://travis-ci.org/koivunej/derive-into-owned)
[![crates.io](https://img.shields.io/crates/v/derive-into-owned.svg)](https://crates.io/crates/derive-into-owned)
[![docs.rs](https://docs.rs/derive-into-owned/badge.svg)](https://docs.rs/derive-into-owned/)

Rust procedural macros for deriving methods to help with working with types that contain [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) fields.
Please note that this derive somewhat strangely works with duck-typing, at least for now.
It was originally created to help me reduce the boilerplate with `Cow` types.

`[derive(IntoOwned)]` generates a method similar to:

```rust
use std::borrow::Cow;

struct Foo<'a> {
	field: Cow<'a, str>,
}

impl<'a> Foo<'a> {
	/// This method would be derived using #[derive(IntoOwned)]
	pub fn into_owned(self) -> Foo<'static> {
		Foo {
			field: Cow::Owned(self.field.into_owned()),
		}
	}
}
```

Originally based off of [deep-clone-derive](https://github.com/asajeffrey/deep-clone/blob/master/deep-clone-derive/lib.rs) example but supports:

 * [tuple structs](./tests/tuple_struct.rs)
 * normal [structs](./tests/struct.rs)
 * enums with tuple variants [tuple enums](./tests/simple_enum.rs)
 * `IntoOwned` alike fields (actually assumes all fields with types with lifetimes are `IntoOwned` alike)
 * [options of Cow or Cow-like types](./tests/opt_field.rs) `Option<Cow<'a, str>>` and `Option<Foo<'a>>`
 * [vectors of Cow or Cow-like types](./tests/vec.rs)

But wait there is even more! `[derive(Borrowed)]` generates a currently perhaps a bit limited version of a method like:

```rust
impl<'a> Foo<'a> {
	pub fn borrowed<'b>(&'b self) -> Foo<'b> {
		Foo {
			field: Cow::Borrowed(self.field.as_ref()),
		}
	}
}
```

## Types with lifetimes

If your struct has a field with type `Bar<'a>` then `Bar` is assumed to have a method `fn into_owned(self) -> Bar<'static>`.

Note, there's no trait implementation expected because I didn't find one at the time and didn't think to create my own, assumed the `Cow::into_owned` might be getting an extension in standard library which never happened and so on.

## Limitations

Currently deriving will fail miserably for at least but not limited to:

 * `IntoOwned`: borrowed fields like `&'a str`
 * `Borrowed`: struct/enum has more than one lifetime
 * both: arrays not supported
 * both: into_owned/borrowed types inside tuples inside vectors

Using with incompatible types results in not so understandable error messages. For example, given a struct:

```rust
#[derive(IntoOwned)]
struct Foo<'a> {
	field: &'a str,
}
```

The compiler error will be:

```
error[E0495]: cannot infer an appropriate lifetime for lifetime parameter `'a` due to conflicting requirements
 --> tests/does_not_compile.rs:4:10
  |
4 | #[derive(IntoOwned)]
  |          ^^^^^^^^^
  |
note: first, the lifetime cannot outlive the lifetime 'a as defined on the impl at 4:10...
 --> tests/does_not_compile.rs:4:10
  |
4 | #[derive(IntoOwned)]
  |          ^^^^^^^^^
note: ...so that reference does not outlive borrowed content
 --> tests/does_not_compile.rs:4:10
  |
4 | #[derive(IntoOwned)]
  |          ^^^^^^^^^
  = note: but, the lifetime must be valid for the static lifetime...
note: ...so that expression is assignable (expected Foo<'static>, found Foo<'_>)
 --> tests/does_not_compile.rs:4:10
  |
4 | #[derive(IntoOwned)]
  |          ^^^^^^^^^
error: aborting due to previous error(s)
```
