# Changelog [![crates.io][crate-badge]][crate] [![docs.rs][docs-badge]][crate]

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

## [Unreleased]

## [1.2.2] - 2022-01-09

### Fixed

- "LLVM ERROR: Invalid LLVMRustVisibility value!" with `1.2.1`/[`1d6f2b9`] by
  making the statics `pub`.

[`1d6f2b9`]: https://github.com/nvzqz/embed-plist-rs/commit/1d6f2b997acb9db76062aaae4c55f8c41422660e

## [1.2.1] - 2022-01-09

### Fixed

- Property section was not being emitted due to a change in how `#[used]` works.
  This seems to be caused by [rust#87570], which switches `#[used]` from
  `llvm.used` to `llvm.compiler.used`.

[rust#87570]: https://github.com/rust-lang/rust/pull/87570

## [1.2.0] - 2020-08-30

### Added

- `embed_info_plist_bytes!` macro.
- `embed_launchd_plist_bytes!` macro.

## [1.1.0] - 2020-08-30

### Added

- `get_info_plist` function to get contents from `embed_info_plist!` macro.
- `get_launchd_plist` function to get contents from `embed_launchd_plist!` macro.

### Changed

- docs.rs only targets `x86_64-apple-darwin`.

### Removed

- Note about reuse test only working on Rust 1.43+. This only applies to doctest
  for some reason.

## 1.0.0 - 2020-08-30

### Added

- `embed_info_plist!` macro.
- `embed_launchd_plist!` macro.

[crate]:       https://crates.io/crates/embed_plist
[crate-badge]: https://img.shields.io/crates/v/embed_plist.svg
[docs]:        https://docs.rs/embed_plist
[docs-badge]:  https://docs.rs/embed_plist/badge.svg

[Keep a Changelog]:    http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

[Unreleased]: https://github.com/nvzqz/embed-plist-rs/compare/v1.2.2...HEAD
[1.2.2]:      https://github.com/nvzqz/embed-plist-rs/compare/v1.2.1...v1.2.2
[1.2.1]:      https://github.com/nvzqz/embed-plist-rs/compare/v1.2.0...v1.2.1
[1.2.0]:      https://github.com/nvzqz/embed-plist-rs/compare/v1.1.0...v1.2.0
[1.1.0]:      https://github.com/nvzqz/embed-plist-rs/compare/v1.0.0...v1.1.0
