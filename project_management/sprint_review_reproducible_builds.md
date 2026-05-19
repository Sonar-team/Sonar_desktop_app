# Sprint Review: Reproducible Builds

## Sprint Goal

Evaluate whether the sprint moved SONAR toward reproducible release builds by
controlling dependency resolution, toolchain versions, build environment inputs,
and release verification.

## Conclusion

The sprint outcome is **effectively achieved** for the intended sprint scope.

The project now has a controlled build baseline:

- Rust, Node.js, Deno, and Tauri CLI versions are centralized in
  `config/build-versions.env`.
- Rust is pinned to an exact toolchain version.
- The Docker build uses a pinned Rust image digest.
- Frontend dependency installation is frozen with `deno install --frozen`.
- Cargo dependency resolution is forced through vendored sources.
- apt package versions and dated Debian/Ubuntu snapshots are documented and
  wired into CI/container setup.
- Reproducibility flags are centralized through `security/repro-env.ts`.
- The release workflow gates publishing behind a Linux reproducibility check.
- Release hashes, signing, provenance, and SBOM are treated as release-trust
  outputs separate from byte-for-byte reproducibility.

This does **not** mean the full reproducible-build objective is complete. The
remaining work is now narrower: packaged artifact reproducibility and release
trust publication.

## Completed Sprint Work

### 1. Toolchain versions are pinned and centralized

Canonical versions are tracked in `config/build-versions.env`:

- Rust `1.95.0`
- Node.js `24.14.0`
- Deno `2.7.13`
- Tauri CLI `2.11.1`

`script/ci/check-build-versions.sh` verifies that key files stay aligned with
this central version file.

### 2. Rust and frontend dependency inputs are controlled

Rust dependencies are resolved from:

- `src-tauri/.cargo/config.toml`
- `src-tauri/vendor/`
- `src-tauri/Cargo.lock`

Frontend dependencies are resolved from:

- `deno.json`
- `deno.lock`
- `package.json`

CI and Docker use frozen installs so dependency graph drift is treated as a
build failure instead of a silent update.

### 3. Docker inputs are mostly deterministic

The Dockerfile no longer uses `rust:latest`. It now uses:

```text
rust:1.95.0@sha256:5b1e3484ddcd22a3738c0ec34a5e98bf19382eb295fb6db54295e62379119040
```

The Docker build also pins Node.js, Deno, and Debian package versions.
The Docker build now also verifies the downloaded Node.js archive against the
published `SHASUMS256.txt` file and the downloaded Deno archive against the
published `.sha256sum` file before extraction.

### 4. OS package inputs are pinned or snapshot-backed

`config/build-versions.env` tracks:

- `APT_SNAPSHOT_TIMESTAMP`
- Debian and Ubuntu snapshot base URLs
- GitHub Actions Ubuntu package pins
- Docker/GitLab Debian package pins

`script/ci/use-apt-snapshot.sh` applies the dated package snapshot before
package installation.

### 5. Reproducibility flags are part of the real release path

`security/repro-env.ts` centralizes:

- `SOURCE_DATE_EPOCH`
- Rust path remapping with `--remap-path-prefix`
- Windows `/Brepro` linker flag support when enabled

The release workflow and supporting checks now consume this shared environment
instead of duplicating flags in separate scripts.

### 6. CI verification exists

The release workflow includes a `reproducibility-check` job before publishing.

Additional diagnostic workflows exist for:

- bundle reproducibility checks
- Windows binary reproducibility investigation
- reproducibility environment wiring

This gives the project a process-level gate for the Linux unsigned target and a
diagnostic path for Windows/macOS.

## Current Limitations

### 1. Packaged outputs are not fully reproducible yet

Linux binary reproducibility is now the strongest target. Packaged artifacts
remain the main open area.

Known follow-up:

- Debian package reproducibility is tracked separately, including issue `#107`.
- Windows NSIS and macOS DMG probes show nondeterminism in bundle outputs.
- The next investigation should compare raw platform binaries separately from
  final installers.

### 2. Some CI inputs remain intentionally diagnostic

Release-oriented runners are pinned more tightly than before:

- `windows-2022`
- `macos-14`
- Linux release check documented as `ubuntu-22.04`

Some non-release workflows still use `ubuntu-latest`. That is acceptable for
general lint/security workflows, but they should not be treated as canonical
reproducibility environments.

### 3. Release trust outputs are not the same as reproducibility

Provenance attestation is now wired into the release workflow. Signing and SBOM
publication remain release trust steps. These outputs should stay outside the
byte-for-byte reproducibility comparison because their metadata is expected to
vary.

Expected trust flow:

1. Validate reproducibility on unsigned artifacts.
2. Build and publish release artifacts.
3. Sign artifacts through CI.
4. Publish signatures, hashes, provenance, and SBOM metadata.

## Sprint Verdict

The sprint is **successful for the sprint outcome**:

- toolchain drift is controlled
- dependency drift is controlled
- Linux release-style reproducibility is enforced
- canonical build assumptions are documented
- future work is clearly scoped

The broader reproducible-build objective remains open until repeated CI builds
produce identical target artifacts for the chosen final scope, including the
packaged outputs that the project decides to enforce.

## Recommended Next Sprint Actions

1. Resolve Debian package nondeterminism tracked by issue `#107`.
2. Split Windows checks into raw binary comparison and NSIS installer
   comparison.
3. Split macOS checks into raw binary, `.app` bundle, and DMG comparison.
4. Keep the Docker bootstrap verification in sync with version bumps for
   Node.js and Deno.
5. Start the release trust track:
   - sign release artifacts in CI
   - verify provenance on the next tagged release
   - generate and publish SBOM artifacts
6. Keep reproducibility validation focused on unsigned artifacts.

## Final Assessment

The sprint moved from a partially controlled build to a documented and
CI-guarded reproducibility baseline.

The next risk is no longer basic toolchain drift. The next risk is packaging
nondeterminism and release trust metadata.
