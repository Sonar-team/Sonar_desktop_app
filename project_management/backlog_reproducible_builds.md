# Backlog: Reproducible Builds

## Goal

Make SONAR builds reproducible by controlling dependency resolution, toolchain
versions, build environment inputs, and release verification.

## Must Have

### 1. Pin the Rust toolchain exactly

Status: Done

- Update `src-tauri/rust-toolchain.toml` to use a fixed Rust version instead of
  `stable`.
- Keep `src-tauri/Cargo.toml` aligned with the same Rust version.

Why:

- prevents compiler drift across rebuilds
- ensures the same source is built with the same compiler version

### 2. Pin Deno everywhere

Status: Done

- Use the same exact Deno version in CI, local build documentation, and
  container builds.
- Remove placeholder or floating Deno version references.

Why:

- prevents frontend toolchain drift
- ensures lockfile resolution is interpreted by the same runtime

### 3. Keep Cargo vendoring enforced

Status: Done

- Keep `src-tauri/.cargo/config.toml` pointing to vendored sources.
- Ensure `src-tauri/vendor/` is always committed and refreshed intentionally.

Why:

- avoids crates.io dependency drift
- supports offline and repeatable Rust builds

### 4. Keep frontend installs frozen

Status: Done

- Keep `deno.json` and `deno.lock` authoritative.
- Fail CI if install would modify the lockfile.

Why:

- avoids silent frontend dependency graph changes
- makes rebuild inputs stable

### 5. Make the Docker build deterministic

Status: Done

- Replace `rust:latest` with a pinned image or digest.
- Stop downloading bootstrap tools from floating branches like GitHub `master`.
- Pin Deno and Node bootstrap sources explicitly.

Recent progress:

- Commit `c3b83ecd` pins the Rust Docker base image by digest.
- Docker now pins Node.js and Deno versions through `config/build-versions.env`.
- The Dockerfile now verifies the downloaded Node.js archive against the
  published `SHASUMS256.txt` file.
- The Dockerfile now verifies the downloaded Deno archive against the
  published `.sha256sum` file.

Why:

- current container inputs can change without any source code change
- deterministic builds require deterministic base environments

### 6. Pin or snapshot OS package dependencies

Status: Done

- Pin apt package versions where practical, or build from a known image
  snapshot.
- Document the expected Linux packaging dependencies.

Implementation notes:

- GitHub Actions Ubuntu package versions are pinned in
  `config/build-versions.env` as `LINUX_APT_PACKAGES`.
- GitLab and Docker Debian package versions are pinned in
  `config/build-versions.env` as `GITLAB_APT_PACKAGES` and `DOCKER_APT_PACKAGES`
  and mirrored in the Dockerfile build argument.
- `script/ci/use-apt-snapshot.sh` configures Debian and Ubuntu jobs to use a
  dated apt snapshot before package installation.
- `script/ci/check-build-versions.sh` validates that the Dockerfile and CI
  workflows keep using the centralized pinned package variables.

Why:

- system package updates can change build outputs
- release packaging often depends on exact system library behavior

### 7. Apply reproducibility flags in real release builds

Status: Done

- Reuse the logic already present in `security/repro-check.sh`.
- Add `SOURCE_DATE_EPOCH` to release-oriented build steps.
- Add path remapping such as `--remap-path-prefix`.
- Validate the release-style build path on `ubuntu-22.04`, `windows-2022`, and
  `macos-14`.

Why:

- avoids timestamp drift
- removes local machine path leakage from build outputs

### 8. Enforce reproducibility verification in CI

Status: Done

- Run `security/repro-check.sh` on release candidates or tags.
- Fail the pipeline when repeated builds are not identical.

Why:

- manual checks are not enough
- the sprint goal needs an enforced quality gate

## Should Have

### 9. Publish SHA256 hashes for release artifacts

Status: Done

- Generate hashes for binaries and packaged artifacts.
- Publish them with release outputs.

Why:

- helps users and developers verify rebuilds
- creates a clear comparison point for reproducibility checks

### 10. Separate signing from reproducibility validation

Status: Done

- Validate reproducibility on unsigned artifacts first.
- Apply signing after reproducibility checks pass.

Implementation notes:

- `security/repro-check.sh` defaults to `deno task tauri build --ci --no-sign`.
- `script/ci/check-bundle-repro.sh` also uses `--no-sign` for NSIS/DMG probes.
- The release workflow gates publishing behind the unsigned reproducibility
  check.
- Signing, provenance, and SBOM remain separate release-trust work after
  reproducibility validation.

Why:

- signatures and signing metadata are intentionally variable
- signed outputs are not the right first-level reproducibility target

### 11. Define the exact reproducibility target

Status: Done

- The canonical target is documented in
  `project_management/canonical_build_environment.md`.
- Reproducibility validation targets unsigned platform binaries and unsigned
  platform bundles.
- Signing, provenance, and SBOM generation are explicitly outside the
  byte-for-byte reproducibility comparison.

Why:

- prevents ambiguity
- helps choose the right scope for validation and acceptance criteria

### 12. Document the canonical build environment

Status: Done

- Write down the expected OS, toolchain, Deno version, Rust version, package
  dependencies, and release process.

Implementation notes:

- `project_management/canonical_build_environment.md` now documents the shared
  baseline, Linux target, Windows target, macOS target, and platform rollout.
- Commit `dac1e733` documents the keyless signing flow.
- Commits `c3b83ecd` and `9cfc3b3b` update the canonical environment with the
  pinned Rust image digest and apt snapshot/package inputs.

Why:

- makes independent rebuilds possible
- reduces tribal knowledge

## Later

### 13. Reduce moving targets in GitHub Actions

Status: Partially Done

- Replace `macos-latest` and `windows-latest` where possible with more stable
  runner choices or documented runner assumptions.
- Review floating action references and pin them more tightly if needed (with a
  hash).

Recent progress:

- Release-oriented Windows and macOS workflows now use `windows-2022` and
  `macos-14`.
- Build versions are exported from `config/build-versions.env` in release and
  diagnostic workflows.

Remaining:

- Some non-release workflows still use `ubuntu-latest`.
- Several actions are still referenced by tag rather than immutable commit SHA.

Why:

- hosted runner environments evolve over time
- stable CI inputs improve repeatability

### 14. Extend reproducibility checks to packaged outputs

Status: Partially Done

- Linux binary and `.deb` package reproducibility are now confirmed.
- Windows NSIS is fixed locally: `check-bundle-repro.sh` now produces identical
  `sonar.exe` and `sonar_3.13.8_x64-setup.exe` hashes across two clean
  `cargo-xwin` builds (`sonar.exe`
  `f7c051ae66d07bfd55a37ad65e860202884bcf3da36b74b0511e967f27e7926e`,
  setup `.exe`
  `e9f8e4d814e25e717e3795f52caa533894bfc25b7b37ecf6c697a4f40c5dd06e`).
- Continue with GitHub validation for Windows NSIS, then RPM, Windows MSI smoke,
  and macOS DMG packaging.
- GitHub Actions probes have been added for Windows NSIS and macOS DMG.
- Commit `e4f7ed6b` makes Windows release binaries reproducible.
- Latest `publish-smoke.yml` run on `main` confirmed
  `verify reproducible Debian package` succeeds for
  `sonar_3.13.8_amd64.deb`; two repackaged `.deb` artifacts produced identical
  SHA256 `e256acced3e8534395d277f84b6b4ef648e232105fc09074a036fe8ac5531b14`.
- Remaining bundle result: RPM and DMG artifacts still need separate tracking
  before they can be treated as enforced reproducibility targets. Windows NSIS
  should move to enforced once the updated workflow is green on GitHub Actions.
- Windows MSI smoke currently fails in WiX `candle.exe` while compiling
  `src-tauri/windows/fragments/npcap.wxs`.
- Next step: keep comparing raw platform binaries separately from final bundles,
  then fix or replace packaging steps that introduce nondeterminism.
- Tracking issue `#107` should be closed or updated if it only covered the
  historical Debian package nondeterminism.

Why:

- raw binaries are easier to stabilize first
- installer formats usually introduce extra nondeterminism
- owning the packaging script may be required if Tauri's bundler does not expose
  enough control over timestamps, metadata, ownership, or file ordering

### 15. Add provenance alongside reproducibility

Status: Partially Done

- Keep artifact signing and provenance generation as a separate but related
  track.
- Use them to prove origin after reproducibility is under control.

Recent progress:

- Commit `dac1e733` documents the keyless release signing flow.
- Commit `d37dd5cb` adds CycloneDX SBOM artifacts.
- Commit `9c2569a1` adds source manifest generation.
- The release workflow now generates GitHub artifact provenance attestations for
  the raw binary, platform bundle, and release hash manifest after each platform
  build.

Remaining:

- Release artifact signing is not yet fully wired as a publishing step.
- SBOM and manifest artifacts still need to be published consistently with
  releases.

Why:

- provenance helps trust the builder
- reproducibility helps trust the result

## Proposed Order

1. Pin Rust exactly.
2. Pin Deno exactly everywhere.
3. Keep vendoring and lockfile enforcement strict.
4. Stabilize Docker and system package inputs.
5. Move reproducibility flags into real release builds.
6. Enforce reproducibility checks in CI.
7. Publish hashes and document the canonical rebuild process.
8. Extend validation to packaged artifacts and provenance.

## Sprint Outcome

The current sprint can reasonably be considered successful when:

- release builds use pinned toolchains and locked dependencies
- reproducibility flags are applied in the real release path
- the release-style CI build works on `ubuntu-22.04`, `windows-2022`, and
  `macos-14`
- a lightweight CI check protects the reproducibility environment wiring

This sprint outcome is now effectively achieved, but it is not the same thing as
the full reproducible-build objective being complete.

## Next Sprint Focus

The next delivery focus should shift toward the remaining packaged artifact
gaps, then supply-chain trust and release authenticity:

- fix Windows MSI smoke around `npcap.wxs` / WiX `candle.exe`
- investigate RPM package nondeterminism
- validate the fixed NSIS setup `.exe` reproducibility path on GitHub Actions
- decide whether DMG should remain a byte-for-byte target or move enforcement
  to the normalized `.app` input root
- sign release artifacts in CI
- verify the published provenance path on the next tagged release
- publish SBOM and source-manifest metadata with release outputs
- keep reproducibility verification as the next enforcement step rather than
  mixing signing into the reproducibility target

Guiding rule:

- reproducibility should target unsigned artifacts first
- signing, provenance, and SBOM should prove origin and content after the build
  completes

Release trust flow:

1. Run reproducibility validation on unsigned artifacts.
2. Build and publish release artifacts.
3. Sign the released binary and bundle artifacts with Sigstore/cosign keyless
   signing on GitHub Actions.
4. Publish signatures, hashes, provenance, and SBOM metadata alongside the
   release artifacts.

The signing job should use GitHub Actions OIDC with `id-token: write`, run only
on trusted release tags such as `app-v*`, and stay out of the reproducibility
comparison path.

## User Stories

### Signing

- As a SONAR user, I want release artifacts to be signed so that I can verify
  they were produced by the official project pipeline.
- As a maintainer, I want signing to happen in CI so that release trust does not
  depend on manual local steps.
- As a maintainer, I want signing keys or identities to be managed through CI
  secrets or trusted signing infrastructure so that releases remain auditable.

### Provenance

- As a security reviewer, I want provenance attached to each release so that I
  can see which workflow, commit, and builder produced the artifact.
- As a downstream integrator, I want provenance generated automatically in CI so
  that I can verify build origin without trusting a handwritten release note.
- As a maintainer, I want provenance to be published alongside the artifact so
  that origin verification is part of the release itself.

### SBOM

- As a SONAR user, I want an SBOM for each release so that I can understand
  which components and dependencies are included.
- As a security team member, I want the SBOM generated from CI for the exact
  release artifact so that dependency inventory matches what was actually
  shipped.
- As a maintainer, I want the SBOM published with the release so that
  vulnerability review and compliance checks can be done without rebuilding
  locally.

## Objective Done

The reproducible-build objective can be considered fulfilled when:

- the toolchain is fully pinned
- dependency resolution is locked and vendored where required
- the release build environment is stable and documented
- repeated CI builds of the same revision produce identical target artifacts for
  the chosen scope
- reproducibility checks are enforced automatically

This objective remains broader than the current sprint and should stay separate
from signing, provenance, and SBOM deliverables, even though all of them
contribute to overall release trust.
