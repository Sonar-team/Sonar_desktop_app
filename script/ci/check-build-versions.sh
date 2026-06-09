#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

eval "$(./script/ci/export-build-versions.sh)"

check_contains() {
  local file="$1"
  local expected="$2"

  if ! grep -Fq "$expected" "$file"; then
    echo "Expected '$expected' in $file" >&2
    exit 1
  fi
}

check_contains src-tauri/rust-toolchain.toml "channel = \"${RUST_VERSION}\""
check_contains package.json "\"node\": \"${NODE_VERSION}\""
check_contains package.json "\"@tauri-apps/cli\": \"${TAURI_CLI_VERSION}\""
check_contains Dockerfile "FROM rust:${RUST_VERSION}@${RUST_IMAGE_DIGEST} AS builder"
check_contains Dockerfile "ARG DOCKER_APT_PACKAGES=\"${DOCKER_APT_PACKAGES}\""
check_contains Dockerfile 'RUN /app/script/ci/use-apt-snapshot.sh'
check_contains Dockerfile 'RUN apt install -y ${DOCKER_APT_PACKAGES}'
check_contains Dockerfile "ENV NODE_VERSION=\"${NODE_VERSION}\""
check_contains Dockerfile "ENV DENO_VERSION=\"${DENO_VERSION}\""
check_contains Dockerfile 'https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt'
check_contains Dockerfile 'sha256sum --check --status node.sha256sum'
check_contains Dockerfile 'https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/${deno_archive}.sha256sum'
check_contains Dockerfile 'sha256sum --check --status "${deno_archive}.sha256sum"'
check_contains .gitlab-ci.yml "image: rust:${RUST_VERSION}"
check_contains .gitlab-ci.yml "NODE_VERSION: ${NODE_VERSION}"
check_contains .gitlab-ci.yml "DENO_VERSION: ${DENO_VERSION}"
check_contains .gitlab/ci/build.yml './script/ci/use-apt-snapshot.sh'
check_contains .gitlab/ci/build.yml 'apt install -y ${GITLAB_APT_PACKAGES}'
check_contains .github/workflows/publish.yml 'sudo ./script/ci/use-apt-snapshot.sh'
check_contains .github/workflows/publish.yml 'apt-get install -y $LINUX_APT_PACKAGES'
check_contains .github/workflows/publish-smoke.yml 'sudo ./script/ci/use-apt-snapshot.sh'
check_contains .github/workflows/publish-smoke.yml 'apt-get install -y $LINUX_APT_PACKAGES'
check_contains .github/workflows/publish-smoke.yml 'debian-repro-smoke:'
check_contains .github/workflows/publish-smoke.yml 'run: deno task tauri build --ci --no-sign --bundles deb'
check_contains .github/workflows/publish-smoke.yml './script/package-deb-repro.sh "$deb_path" "$first_deb"'
check_contains .github/workflows/publish-smoke.yml 'rpm-repro-smoke:'
check_contains .github/workflows/publish-smoke.yml 'BUNDLE_KIND="rpm"'
check_contains .github/workflows/publish-smoke.yml 'ARTIFACT_PATTERN="*.rpm"'
check_contains .github/workflows/publish-smoke.yml 'dmg-repro-smoke:'
check_contains .github/workflows/publish-smoke.yml 'deno task tauri build --ci --no-sign --target "$MACOS_TARGET" --bundles app'
check_contains .github/workflows/publish-smoke.yml './script/package-dmg-repro.sh "$app_path" "${outdir}/${run}.dmg"'
check_contains .github/workflows/publish-smoke.yml 'nsis-repro-smoke:'
check_contains .github/workflows/publish-smoke.yml 'BUNDLE_KIND="nsis"'
check_contains .github/workflows/publish-smoke.yml 'ARTIFACT_PATTERN="*.exe"'
check_contains .github/workflows/publish-smoke.yml 'msi-repro-smoke:'
check_contains .github/workflows/publish-smoke.yml 'deno task tauri build --ci --no-sign --no-bundle'
check_contains .github/workflows/publish-smoke.yml 'script/package-msi-repro.ps1'
check_contains .github/workflows/covecode.yml './script/ci/export-build-versions.sh'
check_contains .github/workflows/covecode.yml 'node-version: "v${{ steps.versions.outputs.NODE_VERSION }}"'

echo "Build version references are aligned with config/build-versions.env"
