#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SBOM_DIR="${SBOM_DIR:-sbom}"

cd "$ROOT_DIR"

mkdir -p "$SBOM_DIR"

if ! command -v cargo-cyclonedx >/dev/null 2>&1; then
  echo "cargo-cyclonedx is required but not installed" >&2
  exit 1
fi

if ! command -v syft >/dev/null 2>&1; then
  echo "syft is required but not installed" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required but not installed" >&2
  exit 1
fi

SONAR_VERSION="$(jq -r '.version' package.json)"
BACKEND_VERSIONED_OUTPUT="$SBOM_DIR/sonar-backend-${SONAR_VERSION}.cdx.json"
FRONTEND_VERSIONED_OUTPUT="$SBOM_DIR/sonar-frontend-${SONAR_VERSION}.cdx.json"
BACKEND_OUTPUT="$SBOM_DIR/sonar-backend.cdx.json"
FRONTEND_OUTPUT="$SBOM_DIR/sonar-frontend.cdx.json"

echo "SONAR version: $SONAR_VERSION"
BACKEND_TEMP="src-tauri/sonar-backend-${SONAR_VERSION}.cdx.json"

echo "Generating backend SBOM: $BACKEND_VERSIONED_OUTPUT"
cargo cyclonedx \
  --manifest-path src-tauri/Cargo.toml \
  --format json \
  --all \
  --target x86_64-unknown-linux-gnu \
  --override-filename "sonar-backend-${SONAR_VERSION}.cdx" \
  --quiet
mv "$BACKEND_TEMP" "$BACKEND_VERSIONED_OUTPUT"
cp "$BACKEND_VERSIONED_OUTPUT" "$BACKEND_OUTPUT"

echo "Generating frontend SBOM: $FRONTEND_VERSIONED_OUTPUT"
SYFT_CHECK_FOR_APP_UPDATE=false syft scan dir:. \
  --exclude './src-tauri/vendor/**' \
  --exclude './src-tauri/target/**' \
  --exclude './node_modules/**' \
  --exclude './.git/**' \
  --exclude './sbom/**' \
  -o cyclonedx-json="$FRONTEND_VERSIONED_OUTPUT"

tmp_frontend="$(mktemp)"
jq '.' "$FRONTEND_VERSIONED_OUTPUT" > "$tmp_frontend"
mv "$tmp_frontend" "$FRONTEND_VERSIONED_OUTPUT"
cp "$FRONTEND_VERSIONED_OUTPUT" "$FRONTEND_OUTPUT"

echo "SBOM artifacts generated:"
echo "  $BACKEND_VERSIONED_OUTPUT"
echo "  $FRONTEND_VERSIONED_OUTPUT"
echo "  $BACKEND_OUTPUT"
echo "  $FRONTEND_OUTPUT"
