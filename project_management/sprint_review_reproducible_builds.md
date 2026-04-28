# Sprint Review: Reproducible Builds

## Sprint Goal

Evaluate whether the recent changes made during this sprint are aligned with the general objective of making the SONAR build reproducible.

## Conclusion

The sprint work is **partially aligned** with the objective.

The strongest changes improve **input reproducibility** and **verification**:

- Rust dependencies are forced through vendored sources in `src-tauri/.cargo/config.toml`.
- Frontend dependency installation is frozen through `deno.json`, `deno.lock`, and `deno install --frozen` in CI.
- Vendored Rust sources were restored under `src-tauri/vendor`.
- A dedicated reproducibility verification script was added in `security/repro-check.sh`.

These are meaningful changes for a reproducibility sprint because they reduce dependency drift and add a way to measure reproducibility directly.

However, the sprint does **not fully achieve** the objective because the **build environment is still not deterministic enough**.

## Changes That Match the Sprint Goal

### 1. Cargo vendoring re-enabled

Commit `ba18169d` restores vendored Cargo resolution in `src-tauri/.cargo/config.toml`.

Why it matters:

- prevents dependency drift from crates.io
- makes Rust builds less dependent on external registry state
- supports offline and repeatable builds

### 2. Frozen frontend dependency installation

Commits such as `4f86a7ce` and `64c649c7` move the project toward deterministic frontend installs.

Why it matters:

- `deno.lock` becomes authoritative
- `deno install --frozen` prevents silent dependency graph changes in CI
- frontend inputs are more stable across runs

### 3. Vendored dependency files restored

Commit `0b157d77` re-adds vendor files that are required for the vendoring strategy to actually work.

Why it matters:

- configuration alone is not enough
- the vendored tree has to exist in the repository
- this supports the reproducibility objective directly

### 4. Reproducibility verification script added

Commit `5949c4db` adds `security/repro-check.sh`.

Why it matters:

- runs multiple builds and compares outputs
- uses `SOURCE_DATE_EPOCH`
- uses `--remap-path-prefix` to avoid leaking local paths into artifacts
- adds an objective verification path instead of relying on assumptions

This is good sprint work because reproducibility needs measurement, not only intent.

## Changes That Are Adjacent But Not Core To The Goal

### 1. Lockfile refresh for vulnerable dependencies

Commit `29b0e11c` regenerates `deno.lock` and removes vulnerable transitive dependencies.

Assessment:

- useful and responsible
- improves dependency hygiene
- only indirectly related to reproducibility

This is more of a security maintenance change than a reproducible-build change.

### 2. Clippy fixes

Commit `d902e701` fixes cross-platform Clippy issues.

Assessment:

- useful for code quality and CI health
- not materially related to deterministic builds

## What Still Blocks The Sprint Objective

### 1. Rust toolchain is not pinned precisely

`src-tauri/rust-toolchain.toml` uses:

- `channel = "stable"`

Why this is a problem:

- `stable` changes over time
- the same source code can be built with different compiler versions on different dates

For a reproducibility sprint, this should be pinned to an exact Rust version.

### 2. Docker build environment is not deterministic

`Dockerfile` still uses moving inputs:

- `FROM rust:latest`
- `node-build` downloaded from GitHub `master`
- global Deno install without a pinned version
- `apt install` without version pinning

Why this is a problem:

- the container can change even if the source code does not
- rebuilds at different times may not use the same compiler, bootstrap tools, or system libraries

This is one of the main reasons the sprint cannot yet claim full reproducibility.

### 3. Release CI still depends on moving targets

`.github/workflows/publish.yml` still uses:

- `macos-latest`
- `windows-latest`
- `dtolnay/rust-toolchain@stable`
- `deno-version: vx.x.x`

Why this is a problem:

- runner images change
- toolchain state changes
- the release workflow is not pinned tightly enough for reproducible release artifacts

### 4. Reproducibility verification is not enforced in CI

The script `security/repro-check.sh` exists, but it is not currently part of the release gate.

Why this matters:

- manual verification is useful
- enforced verification is what turns reproducibility into process discipline

## Sprint Review Verdict

The sprint is **directionally correct** and contains several changes that are clearly aligned with the reproducible-build objective.

The most valuable outputs of the sprint are:

- restoring vendored Rust dependency resolution
- freezing frontend installs
- restoring the vendored dependency tree
- adding a reproducibility check script

But the sprint remains **incomplete** because it does not yet lock the execution environment tightly enough.

## Recommended Next Sprint Actions

1. Pin Rust to an exact version in `src-tauri/rust-toolchain.toml`.
2. Pin Deno to an exact version in CI.
3. Replace `rust:latest` with a pinned image or digest.
4. Stop downloading bootstrap tooling from floating branches like GitHub `master`.
5. Pin or snapshot system packages used in the container and release workflow.
6. Add the reproducibility check script to CI for tagged or release builds.
7. Publish artifact hashes and keep provenance/signing as a separate concern from reproducibility.

## Final Assessment

The work completed during this sprint is **aligned with the sprint purpose in part, but not enough to claim success end-to-end**.

It successfully improves:

- dependency determinism
- vendoring
- reproducibility testing

It does not yet fully solve:

- toolchain determinism
- OS package determinism
- CI environment determinism
- release artifact reproducibility enforcement
