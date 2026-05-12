#!/usr/bin/env bash
set -euo pipefail

: "${BUNDLE_LABEL:?BUNDLE_LABEL is required}"
: "${BUNDLE_DIR:?BUNDLE_DIR is required}"
: "${ARTIFACT_PATTERN:?ARTIFACT_PATTERN is required}"

BUILD_ARGS="${BUILD_ARGS:-}"
BUILD_TARGET="${BUILD_TARGET:-}"
OUTPUT_DIR="${OUTPUT_DIR:-bundle-repro}"

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

run_build() {
  local run="$1"
  local outdir="${OUTPUT_DIR}/${run}"
  local artifact_list="${outdir}/ARTIFACTS"

  rm -rf dist src-tauri/target "$outdir"
  mkdir -p "$outdir"

  if [[ -n "$BUILD_TARGET" && -n "$BUILD_ARGS" ]]; then
    deno task tauri build -- $BUILD_ARGS --target "$BUILD_TARGET"
  elif [[ -n "$BUILD_TARGET" ]]; then
    deno task tauri build -- --target "$BUILD_TARGET"
  elif [[ -n "$BUILD_ARGS" ]]; then
    deno task tauri build -- $BUILD_ARGS
  else
    deno task tauri build
  fi

  find "$BUNDLE_DIR" -maxdepth 1 -type f -name "$ARTIFACT_PATTERN" | sort > "$artifact_list"
  if [[ ! -s "$artifact_list" ]]; then
    echo "No bundle artifact found in $BUNDLE_DIR" >&2
    exit 1
  fi

  while IFS= read -r artifact; do
    local hash
    hash="$(hash_file "$artifact")"
    cp "$artifact" "$outdir/"
    printf '%s  %s\n' "$hash" "$(basename "$artifact")" >> "$outdir/SHA256SUMS"
  done < "$artifact_list"
}

run_build run1
run_build run2

echo "Run 1 artifacts:"
cat "${OUTPUT_DIR}/run1/ARTIFACTS"
echo "Run 2 artifacts:"
cat "${OUTPUT_DIR}/run2/ARTIFACTS"

echo "Run 1 hashes:"
cat "${OUTPUT_DIR}/run1/SHA256SUMS"
echo "Run 2 hashes:"
cat "${OUTPUT_DIR}/run2/SHA256SUMS"

if cmp -s "${OUTPUT_DIR}/run1/SHA256SUMS" "${OUTPUT_DIR}/run2/SHA256SUMS"; then
  echo "Bundle is reproducible for ${BUNDLE_LABEL}"
else
  echo "Bundle is not reproducible for ${BUNDLE_LABEL}" >&2
  diff -u "${OUTPUT_DIR}/run1/SHA256SUMS" "${OUTPUT_DIR}/run2/SHA256SUMS" || true
  exit 1
fi
