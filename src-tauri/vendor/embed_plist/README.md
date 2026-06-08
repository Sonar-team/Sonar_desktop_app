<div align="center">
    <a href="https://github.com/nvzqz/embed-plist-rs">
        <img src="https://raw.githubusercontent.com/nvzqz/embed-plist-rs/main/img/icon.svg?sanitize=true"
             height="200px">
        <h1 style="font-size:2rem;margin-top:0;">embed_plist</h1>
    </a>
    <a href="https://crates.io/crates/embed_plist">
        <img src="https://img.shields.io/crates/v/embed_plist.svg" alt="crates.io">
        <img src="https://img.shields.io/crates/d/embed_plist.svg" alt="downloads">
    </a>
    <a href="https://docs.rs/embed_plist">
        <img src="https://docs.rs/embed_plist/badge.svg" alt="docs.rs">
    </a>
    <a href="https://github.com/nvzqz/embed-plist-rs/actions?query=workflow%3Aci">
        <img src="https://github.com/nvzqz/embed-plist-rs/workflows/ci/badge.svg" alt="build status">
    </a>
    <img src="https://img.shields.io/badge/rustc-^1.39.0-blue.svg" alt="rustc ^1.39.0">
</div>
<br>

Embed an [`Info.plist`] or [`launchd.plist`] file directly in your
executable binary, brought to you by [@NikolaiVazquez]!

If you found this library useful, please consider
[sponsoring me on GitHub](https://github.com/sponsors/nvzqz). ❤️

## Index

1. [Motivation](#motivation)
2. [Usage](#usage)
3. [Minimum Supported Rust Version](#minimum-supported-rust-version)
4. [Multi-Target Considerations](#multi-target-considerations)
5. [Get Embedded Property Lists](#get-embedded-property-lists)
6. [Accidental Reuse Protection](#accidental-reuse-protection)
7. [Implementation](#implementation)
8. [License](#license)

## Motivation

Certain programs require an embedded `Info.plist` or `launchd.plist` file to
work correctly. For example:

- `Info.plist` is needed to obtain certain permissions on macOS 10.15 and
  later.

- `launchd.plist` is needed to make
  [`launchd` daemons and user agents](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html).

Doing this manually with [`include_bytes!`] is cumbersome. To understand
why, see the [implementation](#implementation). This library removes the
headache of doing this.

## Usage

This library is available [on crates.io][crate] and can be used by adding
the following to your project's [`Cargo.toml`]:

```toml
[dependencies]
embed_plist = "1.2"
```

...and this to any source file in your crate:

```rust
embed_plist::embed_info_plist!("Info.plist");

// If making a daemon:
embed_plist::embed_launchd_plist!("launchd.plist");
```

Done! It's that simple. 🙂

See [implementation](#implementation) for details on this sorcery.

## Minimum Supported Rust Version

This library targets <b>1.39</b> as its minimum supported Rust version
(MSRV).

Requiring a newer Rust version is considered a breaking change and will
result in a "major" library version update. In other words: `0.1.z` would
become `0.2.0`, or `1.y.z` would become `2.0.0`.

## Multi-Target Considerations

This library only works for [Mach-O](https://en.wikipedia.org/wiki/Mach-O)
binaries. When building a cross-platform program, these macro calls should
be placed behind a `#[cfg]` to prevent linker errors on other targets.

```rust
#[cfg(target_os = "macos")]
embed_plist::embed_info_plist!("Info.plist");
```

## Get Embedded Property Lists

After using these macros, you can get their contents by calling
[`get_info_plist`] or [`get_launchd_plist`] from anywhere in your program.

We can verify that the result is correct by checking it against reading the
appropriate file at runtime:

```rust
embed_plist::embed_info_plist!("Info.plist");

let embedded_plist = embed_plist::get_info_plist();
let read_plist = std::fs::read("Info.plist")?;

assert_eq!(embedded_plist, read_plist.as_slice());
```

If the appropriate macro has not been called, each function creates a
compile-time error by failing to reference the symbol defined by that macro:

```rust
// This fails to compile:
let embedded_plist = embed_plist::get_info_plist();
```

## Accidental Reuse Protection

Only one copy of `Info.plist` or `launchd.plist` should exist in a binary.
Accidentally embedding them multiple times would break tools that read these
sections.

Fortunately, this library makes reuse a compile-time error! This protection
works even if these macros are reused in different modules.

```rust
// This fails to compile:
embed_plist::embed_info_plist!("Info.plist");
embed_plist::embed_info_plist!("Info.plist");
```

This example produces the following error:

```txt
error: symbol `_EMBED_INFO_PLIST` is already defined
 --> src/main.rs:4:1
  |
4 | embed_plist::embed_info_plist!("Info.plist");
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this error originates in a macro (in Nightly builds, run with -Z macro-backtrace for more info)

error: aborting due to previous error
```

> <b>Warning:</b> Although the name `_EMBED_INFO_PLIST` can be seen here, you
> **should not** reference this symbol with e.g. an `extern "C"` block. I
> reserve the right to change this name in a SemVer-compatible update.

## Implementation

Files are read using [`include_bytes!`]. This normally places data in
`__TEXT,__const`, where immutable data is kept. However, property list data
is expected to be in `__TEXT,__info_plist` or `__TEXT,__launchd_plist`. This
section will explain how I accomplish that.

We begin by reading the file from disk:

```rust
const BYTES: &[u8] = include_bytes!("Info.plist");
```

A naïve approach is to do the following:

```rust
#[used] // Prevent optimizing out
#[link_section = "__TEXT,__info_plist"]
static PLIST: &[u8] = BYTES;
```

This doesn't work because only the slice's pointer and length that are
placed in `__TEXT,__info_plist`. The referenced bytes are still placed in
`__TEXT,__const`.

Instead, we need to arrive at the following:

```rust
#[used]
#[link_section = "__TEXT,__info_plist"]
static PLIST: [u8; N] = *BYTES;
```

We can get `N` by using [`len`]. As of Rust 1.39, it is possible to get the
length of a slice within a `const`.

```rust
const N: usize = BYTES.len();
```

The next step is to dereference the bytes into a `[u8; N]`.

There are two approaches:

1. <span id="approach-1"></span>Call [`include_bytes!`] again.

   This is not the approach used by this library because of concerns about
   compile performance. See the [second approach](#approach-2) for what this
   library does.

   The following is all we need:

   ```rust
   #[used]
   #[link_section = "__TEXT,__info_plist"]
   static PLIST: [u8; N] = *include_bytes!("Info.plist");
   ```

   This works because `include_bytes!` actually returns a `&[u8; N]`. It's
   often used as a `&[u8]` because we don't know the size when calling it.

2. <span id="approach-2"></span>Dereference our current bytes through
   pointer casting.

   This is tricker than the [first approach](#approach-1) (and somewhat
   cursed). If you know me, then it's predictable I'd go this route.

   We can get a pointer to the bytes via [`as_ptr`], which is usable in
   `const`:

   ```rust
   const PTR: *const [u8; N] = BYTES.as_ptr() as *const [u8; N];
   ```

   Unfortunately, this pointer can't be directly dereferenced in Rust 1.39
   (minimum supported version).

   ```rust
   // This fails to compile:
   #[used]
   #[link_section = "__TEXT,__info_plist"]
   static PLIST: [u8; N] = unsafe { *PTR };
   ```

   Instead, we must cast the pointer to a reference.

   You may want to reach for [`transmute`], which was stabilized for use in
   `const` in Rust 1.46. However, earlier versions need to be supported, so
   that is not an option for this library.

   This bitwise cast can be accomplished with a `union`:

   ```rust
   union Transmute {
       from: *const [u8; N],
       into: &'static [u8; N],
   }

   const REF: &[u8; N] = unsafe { Transmute { from: PTR }.into };
   ```

   Finally, we can dereference our bytes:

   ```rust
   #[used]
   #[link_section = "__TEXT,__info_plist"]
   static PLIST: [u8; N] = *REF;
   ```

## License

This project is released under either:

- [MIT License](https://github.com/nvzqz/embed-plist-rs/blob/master/LICENSE-MIT)
- [Apache License (Version 2.0)](https://github.com/nvzqz/embed-plist-rs/blob/master/LICENSE-APACHE)

at your choosing.

[`get_info_plist`]:    https://docs.rs/embed_plist/1.2.2/embed_plist/fn.get_info_plist.html
[`get_launchd_plist`]: https://docs.rs/embed_plist/1.2.2/embed_plist/fn.get_launchd_plist.html

[@NikolaiVazquez]: https://twitter.com/NikolaiVazquez

[`Cargo.toml`]: https://doc.rust-lang.org/cargo/reference/manifest.html
[crate]:        https://crates.io/crates/embed_plist

[`Info.plist`]:    https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Introduction/Introduction.html
[`launchd.plist`]: https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html#//apple_ref/doc/uid/TP40001762-104142

[`include_bytes!`]: https://doc.rust-lang.org/std/macro.include_bytes.html
[`as_ptr`]:         https://doc.rust-lang.org/std/primitive.slice.html#method.as_ptr
[`len`]:            https://doc.rust-lang.org/std/primitive.slice.html#method.len
[`transmute`]:      https://doc.rust-lang.org/std/mem/fn.transmute.html
