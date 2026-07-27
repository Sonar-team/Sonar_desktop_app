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
check_contains .github/workflows/publish.yml 'sudo ./script/ci/apt-install-pinned.sh $LINUX_APT_PACKAGES'
check_contains .github/workflows/publish.yml './script/ci/install-sbom-tools.sh'
check_contains .github/workflows/publish.yml './script/ci/generate-sbom-artifacts.sh'
check_contains .github/workflows/publish-smoke.yml 'sudo ./script/ci/use-apt-snapshot.sh'
check_contains .github/workflows/publish-smoke.yml 'sudo ./script/ci/apt-install-pinned.sh $LINUX_APT_PACKAGES'
check_contains .github/workflows/publish-smoke.yml 'smoke build bundles with Tauri'
check_contains .github/workflows/publish-smoke.yml 'npm run tauri build'
check_contains .github/workflows/publish-smoke.yml './script/ci/validate-windows-release-binary.ps1'
check_contains .github/workflows/publish-smoke.yml './script/ci/check-windows-bundles-no-npcap.ps1'
check_contains .github/workflows/publish-smoke.yml './script/ci/smoke-test-release-binary.sh'
check_contains .github/workflows/covecode.yml './script/ci/export-build-versions.sh'
check_contains .github/workflows/covecode.yml 'node-version: "v${{ steps.versions.outputs.NODE_VERSION }}"'
check_contains .github/workflows/rust-ci.yml 'cargo install cargo-vet --version "${{ steps.versions.outputs.CARGO_VET_VERSION }}" --locked'
check_contains .github/workflows/rust-ci.yml 'cargo vet --locked --frozen --no-minimize-exemptions'
check_contains .github/workflows/rust-ci.yml 'cargo vet --store-path ../src-tauri/supply-chain --locked --frozen --no-minimize-exemptions'
# Npcap est un prérequis téléchargé séparément : le bundle Windows est limité
# à NSIS, dont le hook détecte le runtime et ouvre uniquement le site officiel.
check_contains src-tauri/tauri.windows.conf.json '"targets": ["nsis"]'
check_contains src-tauri/windows/hooks.nsh 'https://npcap.com/#download'
check_contains .github/workflows/publish.yml './script/ci/validate-windows-release-binary.ps1'
check_contains .github/workflows/publish.yml './script/ci/check-windows-bundles-no-npcap.ps1'

bundled_npcap="$(
  find . -type f \
    \( -iname 'npcap*.exe' -o -iname 'winpcap*.exe' \) \
    ! -path './.git/*' \
    ! -path './node_modules/*' \
    ! -path './dist/*' \
    ! -path './src-tauri/target/*' \
    ! -path './sonar-rust/target/*' \
    -print -quit
)"
if [[ -n "$bundled_npcap" ]]; then
  echo "Npcap/WinPcap installer must not be committed or bundled: $bundled_npcap" >&2
  exit 1
fi

echo "Build version references are aligned with config/build-versions.env"
