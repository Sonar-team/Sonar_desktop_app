#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSIONS_FILE="${VERSIONS_FILE:-${ROOT_DIR}/config/build-versions.env}"

if [[ ! -f "$VERSIONS_FILE" ]]; then
  echo "Build versions file not found: $VERSIONS_FILE" >&2
  exit 1
fi

set -a
# shellcheck source=/dev/null
source "$VERSIONS_FILE"
set +a

required_vars=(
  RUST_VERSION
  RUST_IMAGE_DIGEST
  RUST_NIGHTLY_VERSION
  TARPAULIN_IMAGE
  TARPAULIN_IMAGE_DIGEST
  DOCKERFILE_FRONTEND_VERSION
  DOCKERFILE_FRONTEND_DIGEST
  DOCKER_BUILD_PLATFORM
  NODE_VERSION
  DENO_VERSION
  TAURI_CLI_VERSION
  VITE_VERSION
  CARGO_CYCLONEDX_VERSION
  CARGO_VET_VERSION
  CARGO_FUZZ_VERSION
  CARGO_XWIN_VERSION
  XWIN_VERSION
  XWIN_SDK_VERSION
  XWIN_CRT_VERSION
  XWIN_ARCH
  XWIN_HTTP_RETRIES
  SYFT_VERSION
  COSIGN_VERSION
  SIGSTORE_TRUSTED_ROOT_SHA256
  UBUNTU_RUNNER
  WINDOWS_RUNNER
  MACOS_RUNNER
  MACOS_TARGET
  APT_SNAPSHOT_TIMESTAMP
  DEBIAN_SNAPSHOT_BASE_URL
  UBUNTU_SNAPSHOT_BASE_URL
  LINUX_APT_PACKAGES
  GITLAB_APT_PACKAGES
  DOCKER_APT_PACKAGES
  WINDOWS_CROSS_APT_PACKAGES
)

for name in "${required_vars[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    echo "Missing required build version variable: $name" >&2
    exit 1
  fi
done

emit() {
  local output="$1"
  local name

  for name in "${required_vars[@]}"; do
    printf '%s=%s\n' "$name" "${!name}" >> "$output"
  done
}

if [[ -n "${GITHUB_ENV:-}" ]]; then
  emit "$GITHUB_ENV"
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  emit "$GITHUB_OUTPUT"
fi

if [[ -z "${GITHUB_ENV:-}" && -z "${GITHUB_OUTPUT:-}" ]]; then
  for name in "${required_vars[@]}"; do
    printf 'export %s=%q\n' "$name" "${!name}"
  done
fi
