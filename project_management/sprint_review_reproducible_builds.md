# Sprint Review: Reproducible Builds and Node 24 CI

## Review Date

2026-05-27

## Sprint Goal

Move SONAR toward reproducible release builds by controlling dependency
resolution, toolchain versions, build environment inputs, and release
verification.

The sprint also absorbed the CI runtime update to Node.js 24, because GitHub
Actions JavaScript actions now need to run cleanly on the Node 24 runtime.

## Outcome

The sprint outcome is **achieved for the agreed scope**.

The project now has a controlled build baseline:

- Rust, Node.js, Deno, and Tauri CLI versions are centralized in
  `config/build-versions.env`.
- Node.js is pinned to `24.14.0` for project CI wiring.
- GitHub Actions workflows force JavaScript actions onto Node 24 with
  `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`.
- Rust is pinned to `1.95.0`.
- Deno is pinned to `2.7.13`.
- Tauri CLI is pinned to `2.11.1`.
- Frontend dependency installation uses `deno install --frozen`.
- Cargo dependency resolution remains backed by vendored sources.
- Linux apt inputs are pinned through dated Ubuntu/Debian snapshots and package
  variables.
- Reproducibility flags are centralized through `security/repro-env.ts`.

This does **not** mean the full reproducible-build objective is complete. The
remaining work is now focused on packaged artifact reproducibility and release
trust publication.

## Delivered Work

### Node 24 CI Runtime

PR `#122` was merged into `main`:

- PR: `Force GitHub Actions onto Node 24`
- Squash merge commit: `9926f8fe8387d5393d278ef6856eb5fca63cb207`

The migration added or validated:

- `NODE_VERSION=24.14.0` in `config/build-versions.env`
- Node 24 runtime forcing across GitHub Actions workflows
- CI validation through `script/ci/check-build-versions.sh`
- build-version export support through `script/ci/export-build-versions.sh`

The Rust CI follow-ups needed for the migration were also completed:

- `cargo outdated` now bypasses vendored Cargo source configuration only for
  the outdated-dependency probe.
- `cargo udeps` runs with nightly, as required by the tool.
- unused direct Rust dependencies `rayon` and `ouroboros` were removed.
- `src-tauri/deny.toml` was added so `cargo deny check` has explicit advisory,
  license, ban, and source policy.

### Reproducible Build Baseline

The sprint kept the reproducibility baseline intact:

- exact Rust toolchain
- frozen frontend dependency install
- vendored Rust dependency source
- pinned Docker Rust image digest
- pinned Node/Deno bootstrap versions
- pinned or snapshot-backed Linux package inputs
- release-style reproducibility flags

### Smoke Test Validation

`publish-smoke.yml` was rerun manually on `main` after the Node 24 merge:

- Run: `26509144502`
- Ref: `main`
- Commit: `9926f8fe8387d5393d278ef6856eb5fca63cb207`

Passing jobs:

- `publish-smoke (ubuntu-22.04)`
- `publish-smoke (macos-14)`
- `verify reproducible Debian package`
- `verify reproducible MSI package`

The Debian package path is confirmed healthy:

- built artifact: `sonar_3.13.8_amd64.deb`
- reproducibility check rebuilt two `.deb` files with identical SHA256:
  `e256acced3e8534395d277f84b6b4ef648e232105fc09074a036fe8ac5531b14`

## Accepted Out Of Scope

The following checks are accepted as outside the Node 24 migration scope:

- `codecov/project`
- `publish-smoke` packaging failures for Windows, RPM, NSIS, and DMG

Reason:

- the Node 24 migration itself is validated by the workflow/runtime checks
- the remaining failures are packaging or coverage-baseline issues
- packaged artifact reproducibility is already tracked as a broader backlog item

## Remaining Failures

### Windows smoke

`publish-smoke (windows-2022)` compiles the Windows binary, then fails while
bundling MSI through WiX:

- failing tool: `candle.exe`
- failing fragment: `src-tauri/./windows/fragments/npcap.wxs`

This is a Windows packaging issue, not a Node 24 runtime failure.

### RPM reproducibility

The Linux binary is reproducible, but the RPM package is not:

- binary hash is identical across both runs
- `.rpm` hashes differ between run 1 and run 2

This points to nondeterminism introduced by RPM packaging metadata or container
layout.

### NSIS reproducibility

Update after local NSIS focus:

- `check-bundle-repro.sh` now normalizes `SOURCE_DATE_EPOCH`, Rust PE
  metadata, generated NSIS inputs, and the `makensis` invocation.
- Local double-build validation with `cargo-xwin` now reports both
  `sonar.exe` (`f7c051ae66d07bfd55a37ad65e860202884bcf3da36b74b0511e967f27e7926e`)
  and `sonar_3.13.8_x64-setup.exe`
  (`e9f8e4d814e25e717e3795f52caa533894bfc25b7b37ecf6c697a4f40c5dd06e`)
  as reproducible.

GitHub Actions validation is still required before treating NSIS as enforced.

### DMG reproducibility

The macOS app input root is stable, but the DMG container is not:

- normalized input roots are identical
- `hdiutil` produces different DMG hashes on the runner

This remains a macOS packaging-container issue.

## Review Feedback Tracked

Gemini review follow-ups from PR `#122` and duplicate PR `#123` were moved to
issue `#124`.

Tracked follow-ups:

- make workflow discovery in `script/ci/check-build-versions.sh` top-level only
- make the Node 24 workflow assertion less brittle
- review whether `wildcards = "deny"` should be used in `src-tauri/deny.toml`
- verify the advisory ignore `RUSTSEC-2026-0097`
- move the misplaced provenance bullet in
  `project_management/backlog_reproducible_builds.md`

## Sprint Verdict

The sprint is successful for the planned outcome:

- toolchain drift is controlled
- Node 24 CI runtime migration is merged
- Linux `.deb` smoke and reproducibility path pass
- key Rust CI checks pass after the migration
- non-Node packaging failures are clearly isolated
- follow-up review feedback is tracked

The broader reproducible-build objective remains open until the project chooses
and enforces final packaged artifact targets across Linux, Windows, and macOS.

## Recommended Next Sprint

1. Fix Windows MSI smoke around `npcap.wxs` and WiX `candle.exe`.
2. Investigate RPM package nondeterminism by comparing package metadata,
   payload ordering, timestamps, ownership, and compression headers.
3. Validate the fixed NSIS setup reproducibility path on GitHub Actions.
4. Decide whether DMG should be an enforced byte-for-byte target or replaced by
   an `.app`-level reproducibility target plus signed release packaging.
5. Complete issue `#124` review follow-ups.
6. Keep release trust work separate from reproducibility:
   - signing
   - provenance
   - SBOM publication

## Final Assessment

The sprint moved the project from a partially controlled reproducibility setup
to a documented CI-guarded baseline with Node 24 support merged.

The next risk is no longer basic toolchain drift. The next risk is platform
packaging nondeterminism.
