# Canonical Build Environment

## Scope

The canonical reproducibility target is:

- unsigned platform binary
- unsigned platform bundle

Signing, provenance, and SBOM generation happen after reproducibility
validation. They are release trust steps, not part of the byte-for-byte
reproducibility comparison.

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
- Vite version
- frontend lockfile
- Cargo lockfile
- vendored Cargo dependencies
- reproducibility environment variables

The shared build goal is to first prove that the unsigned application binary is
reproducible for a given platform, then prove that the unsigned installer or
bundle for that platform is reproducible.

## Shared Toolchains

Canonical toolchain versions are tracked in `config/build-versions.env`.

- Rust: `RUST_VERSION`
- nightly Rust (coverage and cargo-udeps): `RUST_NIGHTLY_VERSION`
- Tarpaulin image: `TARPAULIN_IMAGE`, digest `TARPAULIN_IMAGE_DIGEST`
- frontend Dockerfile: `DOCKERFILE_FRONTEND_VERSION`, digest `DOCKERFILE_FRONTEND_DIGEST`
- Docker host platform: `DOCKER_BUILD_PLATFORM`
- Node.js: `NODE_VERSION`
- Deno: `DENO_VERSION`
- Tauri CLI: `TAURI_CLI_VERSION`
- Vite: `VITE_VERSION`

Stable Rust is pinned in `src-tauri/rust-toolchain.toml` and
`sonar-rust/rust-toolchain.toml`; the dated nightly is used by the coverage and
cargo-udeps workflows. Node.js is declared in `package.json` under
`engines.node`. Deno is pinned in `Dockerfile`. The Tauri CLI version and Vite
version are pinned in `package.json`.

The Windows cross toolchain is pinned in the same source of truth:

- cargo-xwin: `CARGO_XWIN_VERSION` (installed with its upstream lockfile,
  which currently selects xwin `0.9.0` — pinned by cargo-xwin's own
  `Cargo.lock`, not tracked in `config/build-versions.env`)
- Visual Studio manifest: `XWIN_VERSION`
- Windows SDK: `XWIN_SDK_VERSION`
- MSVC CRT: `XWIN_CRT_VERSION`
- target architecture: `XWIN_ARCH`

`XWIN_SDK_VERSION` and `XWIN_CRT_VERSION` must never be left implicit: xwin
otherwise selects the newest entries in Microsoft's mutable manifest, which
can produce different binaries for the same source commit.

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
- Container base image: `rust:${RUST_VERSION}` (see `config/build-versions.env`)
- Binary target: `src-tauri/target/release/sonar`
- Initial bundle target: Debian `.deb`

The pinned digest is tracked in `config/build-versions.env` as
`RUST_IMAGE_DIGEST`.

### Linux System Packages

The Linux build environment must provide pinned apt package versions. The
current CI package lists are tracked in `config/build-versions.env`:

- `LINUX_APT_PACKAGES` for GitHub Actions Ubuntu jobs
- `GITLAB_APT_PACKAGES` for GitLab Debian jobs
- `DOCKER_APT_PACKAGES` for the Debian-based Rust Docker image

All apt-based build jobs must run `script/ci/use-apt-snapshot.sh` before
installing packages. The snapshot timestamp and base URLs are tracked in
`config/build-versions.env`:

- `APT_SNAPSHOT_TIMESTAMP`
- `DEBIAN_SNAPSHOT_BASE_URL`
- `UBUNTU_SNAPSHOT_BASE_URL`

Current GitHub Actions Ubuntu package pins:

```bash
libwebkit2gtk-4.1-dev=2.50.4-0ubuntu0.22.04.1
libappindicator3-dev=12.10.1+20.10.20200706.1-0ubuntu1
librsvg2-dev=2.52.5+dfsg-3ubuntu0.2
patchelf=0.14.3-1
libpcap-dev=1.10.1-4ubuntu1.22.04.1
```

Current Docker Debian package pins:

```bash
libgtk-3-dev=3.24.49-3
pkg-config=1.8.1-4
libjavascriptcoregtk-4.1-dev=2.52.3-2~deb13u1
libsoup-3.0-dev=3.6.5-3
libwebkit2gtk-4.1-dev=2.52.3-2~deb13u1
libpcap-dev=1.10.5-2
```

Current GitLab Debian package pins match the Docker Debian pins and add:

```bash
xz-utils=5.8.1-1
```

The package version pins and dated snapshot must be bumped together when moving
the canonical Linux build environment forward.

The Dockerfile also verifies the downloaded Node.js archive against the
published `SHASUMS256.txt` file and the downloaded Deno archive against the
published `.sha256sum` file before extraction.

### Linux Canonical Build Commands

Build the frontend:

```bash
deno task build
```

Build the unsigned Tauri binary without relying on Tauri package bundling:

```bash
deno task tauri build -- --no-bundle
```

Run the Linux reproducibility check:

```bash
./security/repro-check.sh
```

### Linux Expected Outputs

Two Linux rebuilds of the same revision in the canonical environment must
produce identical unsigned Linux binaries.

Signed artifacts are explicitly excluded from this comparison because signing
metadata is expected to vary.

## Release Trust Flow

The release trust flow must keep the reproducible payload separate from detached
trust metadata:

1. Validate reproducibility on the unsigned Linux binary.
2. Build the release binaries with the reproducibility environment.
3. Build the native bundles with the Tauri bundler.
4. Sign the released artifacts and release documents with detached signatures.
5. Publish signatures, hashes, provenance, SBOMs, and manifests.

Native installer containers are not part of the byte-for-byte reproducibility
gate. They are generated by Tauri from the tagged source and pinned toolchain,
then covered by release hashes, detached Sigstore signatures, and GitHub
attestations.

The signing step should use Sigstore/cosign keyless signing on GitHub Actions.
GitHub Actions OIDC provides a short-lived workflow identity, so the project
does not need to store a long-lived private signing key in repository secrets.

The signing job should run only after reproducibility validation has passed and
after the release artifacts exist. It should be restricted to trusted release
events, currently tags matching `app-v*`.

Expected job order:

```text
reproducibility-check -> create-release -> publish-tauri -> update-release-body
```

The signing job should have the minimum permissions required:

```yaml
permissions:
  contents: write
  id-token: write
```

Provenance attestations are generated by the release build job with GitHub
Artifact Attestations. The job must run on trusted tag builds and must include
the minimum permissions required by the attestation action:

```yaml
permissions:
  contents: write
  id-token: write
  attestations: write
```

The attested subjects are the raw platform binary, native platform bundles, and
the release hash manifest generated for that platform. The helper script
`script/ci/generate-attestation-subjects.sh` writes the checksum list consumed
by `actions/attest`.

After a tagged release, provenance can be verified from a clean checkout with:

```bash
gh attestation verify <artifact-path> -R Sonar-team/Sonar_desktop_app
```

The initial detached-signature subjects are:

- raw application binary for each platform
- native bundles for each platform
- SBOM files
- provenance or attestation metadata exported as files
- SHA256 release hash manifest
- release/source manifest

Current platform bundle targets:

- Linux: `.deb`
- Windows: NSIS setup `.exe`
- macOS: `.dmg`

The signing command should use blob signing and publish the detached Sigstore
bundle alongside the artifact without modifying the artifact bytes:

```bash
cosign sign-blob --yes \
  --bundle "${artifact}.sigstore.json" \
  "${artifact}"
```

The release workflow should not sign artifacts from pull requests, forks, or
untrusted workflow triggers. If stronger controls are needed, attach the signing
job to a protected GitHub Environment such as `release`.

## Windows Target

- Native OS: Windows Server 2022 (`windows-2022`)
- Architecture: `x86_64`
- Binary target: Tauri Windows executable
- Bundle target: NSIS setup executable generated by Tauri

The reproducibility container also cross-compiles the unsigned binary for
`x86_64-pc-windows-msvc`. Its `windows-toolchain` Docker stage downloads and
splats the pinned CRT/SDK before application sources are copied. That layer can
be reused or exported with `docker save`; the application build itself runs
with `RUN --network=none`, proving that it does not contact the Microsoft CDN.
`RUSTUP_TOOLCHAIN` also selects the exact compiler already provided by the
pinned Rust image before `rust-toolchain.toml` is copied, so rustup does not try
to query its distribution server from that offline layer.
Whole-operation retries are implemented by
`script/ci/cache-xwin-toolchain.sh`. If BuildKit DNS is broken on a local
network, the proof helper accepts the explicit `NETWORK=host` override. On the
buildx path it requests the `network.host` entitlement explicitly; the local
BuildKit daemon must also permit that entitlement.

Bootstrapping the `windows-toolchain` stage remains an online operation: an
empty cache must still reach the Visual Studio 17 manifest and Microsoft's
CDN. The selected SDK and CRT are exact and xwin verifies their payloads, but
retries cannot make a long outage hermetic. Network isolation only proves that
the application rebuild is offline after the toolchain has been cached. Issue
#174 must therefore stay open until the same-commit cross-machine proof runs.

Generate a local reference for a precise commit:

```bash
REF=<commit> RUNS=2 NO_CACHE=1 PLATFORM=windows OUT=repro-windows \
  ./script/release/repro-build-container.sh
awk '$2 == "./windows/sonar.exe" { print $1 }' \
  repro-windows/run1/SHA256SUMS
```

Dispatch `.github/workflows/repro-container.yml` at that same commit and pass
the resulting value as `windows_reference_sha256`. Two independent hosted
runners each perform two application builds; the comparison job requires all
four CI hashes to match and, when supplied, requires the local hash to match as
well.

The release smoke workflow verifies that Tauri can build the Windows NSIS
bundle, that no Npcap/WinPcap installer is embedded, and that the generated
`sonar.exe` is a valid x86_64 PE importing `wpcap.dll`. A real
Windows startup smoke remains pending while Npcap usage on hosted runners is
clarified with Nmap in issue #138. NSIS container reproducibility is not
enforced.

## macOS Target

- OS: macOS runner version to be pinned
- Architecture: `aarch64` and/or `x86_64`
- Binary target: Tauri macOS app binary
- Bundle target: `.dmg` generated by Tauri

The release smoke workflow verifies that Tauri can build the macOS bundle. DMG
container reproducibility is not enforced.

## Platform Rollout

1. Enforce reproducibility for the Linux unsigned binary.
2. Publish native bundles from the Tauri bundler.
3. Cover binaries and bundles with hashes, attestations, and detached
   signatures.
