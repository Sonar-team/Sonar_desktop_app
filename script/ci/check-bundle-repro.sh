#!/usr/bin/env bash
set -euo pipefail

: "${BUNDLE_LABEL:?BUNDLE_LABEL is required}"
: "${BUNDLE_DIR:?BUNDLE_DIR is required}"
: "${ARTIFACT_PATTERN:?ARTIFACT_PATTERN is required}"

BUILD_ARGS="${BUILD_ARGS:-}"
BUILD_TARGET="${BUILD_TARGET:-}"
BUNDLE_FALLBACK_DIRS="${BUNDLE_FALLBACK_DIRS:-}"
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
  local search_dirs="${BUNDLE_DIR}"

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

  if [[ -n "$BUNDLE_FALLBACK_DIRS" ]]; then
    search_dirs="${search_dirs}:${BUNDLE_FALLBACK_DIRS}"
  fi

  : > "$artifact_list"
  IFS=':' read -r -a bundle_dirs <<< "$search_dirs"
  for dir in "${bundle_dirs[@]}"; do
    if [[ -d "$dir" ]]; then
      find "$dir" -maxdepth 1 -type f -name "$ARTIFACT_PATTERN" >> "$artifact_list"
    fi
  done
  sort -u "$artifact_list" -o "$artifact_list"

  if [[ ! -s "$artifact_list" ]]; then
    echo "No bundle artifact found for pattern $ARTIFACT_PATTERN" >&2
    echo "Searched bundle directories:" >&2
    printf '  %s\n' "${bundle_dirs[@]}" >&2
    echo "Available bundle-like files under src-tauri/target:" >&2
    find src-tauri/target -type f \( -name '*.dmg' -o -name '*.exe' -o -name '*.msi' -o -name '*.deb' -o -name '*.rpm' \) 2>/dev/null | sort >&2 || true
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
