# Backlog: Reproducible Builds

## Goal

Make SONAR builds reproducible by controlling dependency resolution, toolchain versions, build environment inputs, and release verification.

## Must Have

### 1. Pin the Rust toolchain exactly

- Update `src-tauri/rust-toolchain.toml` to use a fixed Rust version instead of `stable`.
- Keep `src-tauri/Cargo.toml` aligned with the same Rust version.

Why:

- prevents compiler drift across rebuilds
- ensures the same source is built with the same compiler version

### 2. Pin Deno everywhere

- Use the same exact Deno version in CI, local build documentation, and container builds.
- Remove placeholder or floating Deno version references.

Why:

- prevents frontend toolchain drift
- ensures lockfile resolution is interpreted by the same runtime

### 3. Keep Cargo vendoring enforced

- Keep `src-tauri/.cargo/config.toml` pointing to vendored sources.
- Ensure `src-tauri/vendor/` is always committed and refreshed intentionally.

Why:

- avoids crates.io dependency drift
- supports offline and repeatable Rust builds

### 4. Keep frontend installs frozen

- Keep `deno.json` and `deno.lock` authoritative.
- Fail CI if install would modify the lockfile.

Why:

- avoids silent frontend dependency graph changes
- makes rebuild inputs stable

### 5. Make the Docker build deterministic

- Replace `rust:latest` with a pinned image or digest.
- Stop downloading bootstrap tools from floating branches like GitHub `master`.
- Pin Deno and Node bootstrap sources explicitly.

Why:

- current container inputs can change without any source code change
- deterministic builds require deterministic base environments

### 6. Pin or snapshot OS package dependencies

- Pin apt package versions where practical, or build from a known image snapshot.
- Document the expected Linux packaging dependencies.

Why:

- system package updates can change build outputs
- release packaging often depends on exact system library behavior

### 7. Apply reproducibility flags in real release builds

- Reuse the logic already present in `security/repro-check.sh`.
- Add `SOURCE_DATE_EPOCH` to release-oriented build steps.
- Add path remapping such as `--remap-path-prefix`.

Why:

- avoids timestamp drift
- removes local machine path leakage from build outputs

### 8. Enforce reproducibility verification in CI

- Run `security/repro-check.sh` on release candidates or tags.
- Fail the pipeline when repeated builds are not identical.

Why:

- manual checks are not enough
- the sprint goal needs an enforced quality gate

## Should Have

### 9. Publish SHA256 hashes for release artifacts

- Generate hashes for binaries and packaged artifacts.
- Publish them with release outputs.

Why:

- helps users and developers verify rebuilds
- creates a clear comparison point for reproducibility checks

### 10. Separate signing from reproducibility validation

- Validate reproducibility on unsigned artifacts first.
- Apply signing after reproducibility checks pass.

Why:

- signatures and signing metadata are intentionally variable
- signed outputs are not the right first-level reproducibility target

### 11. Define the exact reproducibility target

- Decide whether the target is:
  - same binary on same machine
  - same binary across machines
  - same packaged installers
  - same release artifacts after CI rebuild

Why:

- prevents ambiguity
- helps choose the right scope for validation and acceptance criteria

### 12. Document the canonical build environment

- Write down the expected OS, toolchain, Deno version, Rust version, package dependencies, and release process.

Why:

- makes independent rebuilds possible
- reduces tribal knowledge

## Later

### 13. Reduce moving targets in GitHub Actions

- Replace `macos-latest` and `windows-latest` where possible with more stable runner choices or documented runner assumptions.
- Review floating action references and pin them more tightly if needed.

Why:

- hosted runner environments evolve over time
- stable CI inputs improve repeatability

### 14. Extend reproducibility checks to packaged outputs

- Start with the Linux binary and `.deb`.
- Later extend to Windows and macOS packaging if feasible.

Why:

- raw binaries are easier to stabilize first
- installer formats usually introduce extra nondeterminism

### 15. Add provenance alongside reproducibility

- Keep artifact signing and provenance generation as a separate but related track.
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

## Definition of Done

The reproducible-build objective can be considered fulfilled when:

- the toolchain is fully pinned
- dependency resolution is locked and vendored where required
- the release build environment is stable and documented
- repeated CI builds of the same revision produce identical target artifacts for the chosen scope
- reproducibility checks are enforced automatically
