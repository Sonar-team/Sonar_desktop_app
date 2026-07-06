#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSIONS_FILE="${VERSIONS_FILE:-${ROOT_DIR}/config/build-versions.env}"

# shellcheck source=/dev/null
source "$VERSIONS_FILE"

: "${CARGO_CYCLONEDX_VERSION:?CARGO_CYCLONEDX_VERSION is required}"
: "${SYFT_VERSION:?SYFT_VERSION is required}"

if ! command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo install cargo-cyclonedx --version "${CARGO_CYCLONEDX_VERSION}" --locked
fi

if ! command -v syft >/dev/null 2>&1; then
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  archive="syft_${SYFT_VERSION}_linux_amd64.tar.gz"
  base_url="https://github.com/anchore/syft/releases/download/v${SYFT_VERSION}"

  curl -sSfL -o "${tmpdir}/${archive}" "${base_url}/${archive}"
  curl -sSfL -o "${tmpdir}/checksums.txt" \
    "${base_url}/syft_${SYFT_VERSION}_checksums.txt"

  (
    cd "$tmpdir"
    grep " ${archive}\$" checksums.txt | sha256sum --check --status
  )

  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir" syft
  sudo install -m 0755 "${tmpdir}/syft" /usr/local/bin/syft
fi

cargo cyclonedx --version
syft version
