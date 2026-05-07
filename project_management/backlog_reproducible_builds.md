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

Status: Partially Done

- Replace `rust:latest` with a pinned image or digest.
- Stop downloading bootstrap tools from floating branches like GitHub `master`.
- Pin Deno and Node bootstrap sources explicitly.

Why:

- current container inputs can change without any source code change
- deterministic builds require deterministic base environments

### 6. Pin or snapshot OS package dependencies

Status: Partially Done

- Pin apt package versions where practical, or build from a known image
  snapshot.
- Document the expected Linux packaging dependencies.

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

Status: Not Done

- Validate reproducibility on unsigned artifacts first.
- Apply signing after reproducibility checks pass.

Why:

- signatures and signing metadata are intentionally variable
- signed outputs are not the right first-level reproducibility target

### 11. Define the exact reproducibility target

Status: Not Done

- Decide whether the target is:
  - same binary on same machine
  - same binary across machines
  - same packaged installers
  - same release artifacts after CI rebuild

Why:

- prevents ambiguity
- helps choose the right scope for validation and acceptance criteria

### 12. Document the canonical build environment

Status: Partially Done

- Write down the expected OS, toolchain, Deno version, Rust version, package
  dependencies, and release process.

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

Why:

- hosted runner environments evolve over time
- stable CI inputs improve repeatability

### 14. Extend reproducibility checks to packaged outputs

Status: Not Done

- Start with the Linux binary and `.deb`.
- Later extend to Windows and macOS packaging if feasible.
- Tracking issue: `#107` Debian package is not reproducible across rebuilds.

Why:

- raw binaries are easier to stabilize first
- installer formats usually introduce extra nondeterminism

### 15. Add provenance alongside reproducibility

Status: Not Done

- Keep artifact signing and provenance generation as a separate but related
  track.
- Use them to prove origin after reproducibility is under control.

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

The next delivery focus should shift toward supply-chain trust and release
authenticity:

- sign release artifacts in CI
- generate and publish provenance for release builds
- generate and publish an SBOM for release outputs
- keep reproducibility verification as the next enforcement step rather than
  mixing signing into the reproducibility target

Guiding rule:

- reproducibility should target unsigned artifacts first
- signing, provenance, and SBOM should prove origin and content after the build
  completes

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
