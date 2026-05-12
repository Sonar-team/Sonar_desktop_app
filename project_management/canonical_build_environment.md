# Canonical Build Environment

## Scope

The canonical reproducibility target is:

- unsigned platform binary
- unsigned platform bundle

Signing, provenance, and SBOM generation happen after reproducibility validation.
They are release trust steps, not part of the byte-for-byte reproducibility
comparison.

The application is cross-platform. Reproducibility must therefore be documented
as a shared baseline plus one platform-specific target per operating system.

Initial enforcement starts with Linux because it is the easiest environment to
control in CI. Windows and macOS should follow the same model once their
packaging paths are understood.

## Shared Baseline

These inputs must stay consistent across all platforms:

- source revision
- Rust version
- Node.js version
- Deno version
- Tauri CLI version
- frontend lockfile
- Cargo lockfile
- vendored Cargo dependencies
- reproducibility environment variables

The shared build goal is to first prove that the unsigned application binary is
reproducible for a given platform, then prove that the unsigned installer or
bundle for that platform is reproducible.

## Shared Toolchains

Canonical toolchain versions are tracked in `config/build-versions.env`.

- Rust: `1.95.0`
- Node.js: `24.14.0`
- Deno: `2.7.13`
- Tauri CLI: `2.11.1`

Rust is pinned in `src-tauri/rust-toolchain.toml`.
Node.js is declared in `package.json` under `engines.node`.
Deno is pinned in `Dockerfile`.
The Tauri CLI version is pinned in `package.json`.

When bumping one of these versions, update `config/build-versions.env` first,
then keep the files above aligned. CI validates the alignment with
`script/ci/check-build-versions.sh`.

## Shared Dependency Sources

Rust dependencies must be resolved from the vendored source tree:

- `src-tauri/.cargo/config.toml`
- `src-tauri/vendor/`
- `src-tauri/Cargo.lock`

Frontend dependencies must be resolved from the checked-in lockfile:

- `deno.json`
- `deno.lock`
- `package.json`

CI must fail if dependency installation modifies a lockfile.

## Shared Reproducibility Inputs

Release-style reproducibility checks must run with:

```bash
SOURCE_DATE_EPOCH=<fixed timestamp>
RUSTFLAGS="--remap-path-prefix=<workspace>=/workspace"
```

The shared source of these settings is `security/repro-env.ts`.

## Linux Target

- OS: Ubuntu 22.04 for the Linux reproducibility check
- Architecture: `x86_64` / `amd64`
- Container base image: `rust:1.95.0`
- Binary target: `src-tauri/target/release/sonar`
- Initial bundle target: Debian `.deb`

The base image should eventually be pinned by digest once the release container
is finalized.

### Linux System Packages

The Linux build environment must provide these apt packages:

```bash
libwebkit2gtk-4.1-dev
libappindicator3-dev
librsvg2-dev
patchelf
libpcap-dev
```

The current CI package list is tracked in `config/build-versions.env` as
`LINUX_APT_PACKAGES`.

Package versions should be pinned or sourced from a documented OS snapshot when
the release container is finalized.

### Linux Canonical Build Commands

Build the frontend:

```bash
deno task build
```

Build the unsigned Tauri binary without relying on Tauri package bundling:

```bash
deno task tauri build -- --no-bundle
```

Build the reproducible Debian package:

```bash
script/package-deb-repro.sh
```

The Debian packaging script is expected to create the package from a normalized
package root, with deterministic ownership, permissions, file ordering, and
timestamps derived from `SOURCE_DATE_EPOCH`.

Run the Linux reproducibility check:

```bash
./security/repro-check.sh
```

### Linux Expected Outputs

Two Linux rebuilds of the same revision in the canonical environment must
produce:

- identical unsigned Linux binaries
- identical unsigned Debian packages

Signed artifacts are explicitly excluded from this comparison because signing
metadata is expected to vary.

## Windows Target

- OS: Windows runner version to be pinned
- Architecture: `x86_64`
- Binary target: Tauri Windows executable
- Bundle target: NSIS installer

The first NSIS reproducibility probe runs through the manual GitHub Actions
workflow `.github/workflows/bundle-repro-check.yml`. It builds the same revision
twice on `windows-2022` and compares the generated installer hashes.

The hash comparison logic lives in `script/ci/check-bundle-repro.sh`.

Before enforcing this target, document the exact runner image, Windows
toolchain inputs, installer tooling, and whether NSIS can produce deterministic
output with normalized timestamps and metadata.

## macOS Target

- OS: macOS runner version to be pinned
- Architecture: `aarch64` and/or `x86_64`
- Binary target: Tauri macOS app binary
- Bundle target: `.dmg`

The first DMG reproducibility probe runs through the manual GitHub Actions
workflow `.github/workflows/bundle-repro-check.yml`. It builds the same revision
twice on `macos-14` for `x86_64-apple-darwin` and compares the generated DMG
hashes.

The hash comparison logic lives in `script/ci/check-bundle-repro.sh`.

Before enforcing this target, document the exact runner image, Xcode/toolchain
inputs, code signing boundary, and whether the unsigned `.app` or `.dmg` can be
made deterministic.

## Platform Rollout

1. Enforce reproducibility for the Linux unsigned binary.
2. Enforce reproducibility for the Linux unsigned Debian package.
3. Document and test the Windows unsigned binary.
4. Document and test the Windows unsigned NSIS installer.
5. Document and test the macOS unsigned app bundle.
6. Document and test the macOS unsigned `.dmg`.

Each platform can have its own packaging implementation if the Tauri bundler
does not expose enough control over timestamps, archive metadata, ownership, or
file ordering.
