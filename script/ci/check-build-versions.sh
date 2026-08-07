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
check_contains sonar-rust/rust-toolchain.toml "channel = \"${RUST_VERSION}\""
check_contains src-tauri/Cargo.toml "rust-version = \"${RUST_VERSION}\""
check_contains sonar-rust/Cargo.toml "rust-version = \"${RUST_VERSION}\""
check_contains sonar-rust/crates/sonar-flows-core/README.md "requires Rust ${RUST_VERSION}"
# La copie vendored de sonar-flows-core reste l'archive crates.io immuable :
# son rust-version est un MSRV de dépendance, pas le toolchain de build.
check_contains package.json "\"node\": \"${NODE_VERSION}\""
check_contains package.json "\"@tauri-apps/cli\": \"${TAURI_CLI_VERSION}\""
check_contains package.json "\"vite\": \"${VITE_VERSION}\""
check_contains Dockerfile "# syntax=docker/dockerfile:${DOCKERFILE_FRONTEND_VERSION}@${DOCKERFILE_FRONTEND_DIGEST}"
check_contains Dockerfile "FROM rust:${RUST_VERSION}@${RUST_IMAGE_DIGEST} AS build-base"
check_contains Dockerfile "ENV RUSTUP_TOOLCHAIN=\"${RUST_VERSION}\""
check_contains script/release/repro-build-container.sh 'args=(--platform "$docker_platform"'
check_contains Dockerfile "ARG DOCKER_APT_PACKAGES=\"${DOCKER_APT_PACKAGES}\""
check_contains Dockerfile 'RUN /app/script/ci/use-apt-snapshot.sh'
check_contains Dockerfile 'RUN apt install -y ${DOCKER_APT_PACKAGES}'
check_contains Dockerfile "ENV NODE_VERSION=\"${NODE_VERSION}\""
check_contains Dockerfile "ENV DENO_VERSION=\"${DENO_VERSION}\""
check_contains Dockerfile 'https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt'
check_contains Dockerfile 'sha256sum --check --status node.sha256sum'
check_contains Dockerfile 'https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/${deno_archive}.sha256sum'
check_contains Dockerfile 'sha256sum --check --status "${deno_archive}.sha256sum"'
check_contains Dockerfile "ARG WINDOWS_CROSS_APT_PACKAGES=\"${WINDOWS_CROSS_APT_PACKAGES}\""
check_contains Dockerfile "ARG CARGO_XWIN_VERSION=${CARGO_XWIN_VERSION}"
check_contains Dockerfile "ARG XWIN_VERSION=${XWIN_VERSION}"
check_contains Dockerfile "ARG XWIN_SDK_VERSION=${XWIN_SDK_VERSION}"
check_contains Dockerfile "ARG XWIN_CRT_VERSION=${XWIN_CRT_VERSION}"
check_contains Dockerfile "ARG XWIN_ARCH=${XWIN_ARCH}"
check_contains Dockerfile "ARG XWIN_HTTP_RETRIES=${XWIN_HTTP_RETRIES}"
check_contains Dockerfile 'RUN bash /usr/local/bin/cache-xwin-toolchain.sh'
check_contains Dockerfile 'RUN --network=none deno run -A ./security/repro-env.ts run deno task tauri build'
check_contains .gitlab-ci.yml "image: rust:${RUST_VERSION}@${RUST_IMAGE_DIGEST}"
check_contains .gitlab-ci.yml "NODE_VERSION: ${NODE_VERSION}"
check_contains .gitlab-ci.yml "DENO_VERSION: ${DENO_VERSION}"
check_contains .gitlab/ci/build.yml './script/ci/use-apt-snapshot.sh'
check_contains .gitlab/ci/build.yml 'apt install -y ${GITLAB_APT_PACKAGES}'
check_contains .github/workflows/publish.yml 'sudo ./script/ci/use-apt-snapshot.sh'
check_contains .github/workflows/publish.yml 'sudo ./script/ci/apt-install-pinned.sh $LINUX_APT_PACKAGES'
check_contains .github/workflows/publish.yml './script/ci/install-sbom-tools.sh'
check_contains .github/workflows/publish.yml './script/ci/generate-sbom-artifacts.sh'
check_contains .github/workflows/publish.yml 'cosign-release: "v${{ steps.versions.outputs.COSIGN_VERSION }}"'
check_contains .github/workflows/publish.yml './script/ci/create-offline-verification-kit.sh'
printf '%s  %s\n' \
  "$SIGSTORE_TRUSTED_ROOT_SHA256" \
  security/sigstore-trusted-root.json | sha256sum --check --status
check_contains .github/workflows/publish-smoke.yml 'sudo ./script/ci/use-apt-snapshot.sh'
check_contains .github/workflows/publish-smoke.yml 'sudo ./script/ci/apt-install-pinned.sh $LINUX_APT_PACKAGES'
# L'essai à blanc doit exercer la MÊME commande de build que la publication
# (#136) : un seul build, bundles compris, dans l'environnement épinglé. Un
# smoke qui construirait autrement pourrait passer au vert pendant que la
# release réelle échoue — c'est précisément le défaut corrigé par #136.
check_contains .github/workflows/publish-smoke.yml \
  "deno run -A ./security/repro-env.ts run bash -lc 'deno task tauri build --ci --no-sign \${TAURI_BUILD_ARGS}'"
check_contains .github/workflows/publish.yml \
  "deno run -A ./security/repro-env.ts run bash -lc 'deno task tauri build --ci --no-sign \${TAURI_BUILD_ARGS}'"
# Les preuves d'inclusion tournent dans les deux chaînes, par format.
# La normalisation des dates avant empaquetage (#119) est un crochet Tauri :
# elle doit rester câblée, sinon les installateurs redeviennent non
# reproductibles sans que rien ne le signale.
check_contains src-tauri/tauri.conf.json '"beforeBundleCommand": "bash ./script/ci/normalize-bundle-mtimes.sh"'
check_contains .github/workflows/publish-smoke.yml './script/ci/verify-deb-embeds-binary.sh'
check_contains .github/workflows/publish.yml './script/ci/verify-deb-embeds-binary.sh'
check_contains .github/workflows/publish-smoke.yml './script/ci/verify-macos-dmg-embeds-binary.sh'
check_contains .github/workflows/publish.yml './script/ci/verify-macos-dmg-embeds-binary.sh'
check_contains .github/workflows/publish-smoke.yml './script/ci/package-macos-dmg.sh'
check_contains .github/workflows/publish.yml './script/ci/package-macos-dmg.sh'
check_contains .github/workflows/publish-smoke.yml './script/ci/validate-windows-release-binary.ps1'
check_contains .github/workflows/publish-smoke.yml './script/ci/check-windows-bundles-no-npcap.ps1'
check_contains .github/workflows/publish-smoke.yml './script/ci/smoke-test-release-binary.sh'
check_contains .github/workflows/covecode.yml './script/ci/export-build-versions.sh'
check_contains .github/workflows/covecode.yml 'node-version: "v${{ steps.versions.outputs.NODE_VERSION }}"'
check_contains .github/workflows/covecode.yml "image: ${TARPAULIN_IMAGE}@${TARPAULIN_IMAGE_DIGEST}"
check_contains .github/workflows/covecode.yml 'cargo "+${RUST_NIGHTLY_VERSION}" tarpaulin'
check_contains .github/workflows/rust-ci.yml 'rustup toolchain install "${RUST_NIGHTLY_VERSION}" --profile minimal'
check_contains .github/workflows/rust-ci.yml 'cargo "+${RUST_NIGHTLY_VERSION}" udeps --all-targets --locked'
check_contains .github/workflows/rust-ci.yml 'cargo install cargo-vet --version "${{ steps.versions.outputs.CARGO_VET_VERSION }}" --locked'
check_contains .github/workflows/rust-ci.yml 'cargo install cargo-fuzz --version "${{ steps.versions.outputs.CARGO_FUZZ_VERSION }}" --locked'
check_contains .github/workflows/rust-ci.yml 'cargo vet --locked --frozen --no-minimize-exemptions'
check_contains .github/workflows/rust-ci.yml 'cargo vet --store-path ../src-tauri/supply-chain --locked --frozen --no-minimize-exemptions'
check_contains .github/workflows/repro-container.yml 'machine: [machine-a, machine-b]'
check_contains .github/workflows/repro-container.yml './script/ci/compare-repro-windows-hashes.sh repro-hashes-windows-cross'
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
