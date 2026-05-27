#!/usr/bin/env bash
set -euo pipefail

: "${BUNDLE_LABEL:?BUNDLE_LABEL is required}"
: "${BUNDLE_DIR:?BUNDLE_DIR is required}"
: "${ARTIFACT_PATTERN:?ARTIFACT_PATTERN is required}"

BUILD_ARGS="${BUILD_ARGS:-}"
BUILD_TARGET="${BUILD_TARGET:-}"
BUNDLE_KIND="${BUNDLE_KIND:-}"
BUNDLE_FALLBACK_DIRS="${BUNDLE_FALLBACK_DIRS:-}"
BINARY_DIR="${BINARY_DIR:-}"
BINARY_FALLBACK_DIRS="${BINARY_FALLBACK_DIRS:-}"
BINARY_PATTERN="${BINARY_PATTERN:-}"
OUTPUT_DIR="${OUTPUT_DIR:-bundle-repro}"
MAKENSIS_WRAPPER_DIR=""

append_flag() {
  local existing="$1"
  local flag="$2"

  if [[ " ${existing} " == *" ${flag} "* ]]; then
    printf '%s' "$existing"
  elif [[ -n "$existing" ]]; then
    printf '%s %s' "$existing" "$flag"
  else
    printf '%s' "$flag"
  fi
}

append_link_arg() {
  local existing="$1"
  local link_arg="$2"

  if [[ " ${existing} " == *" ${link_arg} "* ]]; then
    printf '%s' "$existing"
  elif [[ -n "$existing" ]]; then
    printf '%s -C %s' "$existing" "$link_arg"
  else
    printf '%s %s' "-C" "$link_arg"
  fi
}

ensure_repro_env() {
  if [[ -z "${SOURCE_DATE_EPOCH:-}" ]]; then
    SOURCE_DATE_EPOCH="$(git log -1 --format=%ct HEAD)"
    export SOURCE_DATE_EPOCH
  fi

  local remap_flag="--remap-path-prefix=${PWD}=/workspace"
  RUSTFLAGS="$(append_flag "${RUSTFLAGS:-}" "$remap_flag")"

  if [[ "$BUNDLE_KIND" == "nsis" || "$BUILD_TARGET" == *"windows-msvc"* ]]; then
    RUSTFLAGS="$(append_link_arg "$RUSTFLAGS" "link-arg=/Brepro")"
  fi

  export RUSTFLAGS
}

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

normalize_nsis_exe_for_hash() {
  local input="$1"
  local output="$2"
  local source_date_epoch="${SOURCE_DATE_EPOCH:-0}"

  cp "$input" "$output"

  python3 - "$output" "$source_date_epoch" <<'PY'
import struct
import sys

path = sys.argv[1]
source_date_epoch = int(sys.argv[2]) if len(sys.argv) > 2 else 0

with open(path, "r+b") as f:
    data = bytearray(f.read())

    if len(data) < 0x40:
        sys.exit(0)

    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 4 > len(data):
        sys.exit(0)
    if data[pe_offset:pe_offset + 4] != b"PE\0\0":
        sys.exit(0)

    coff_header_offset = pe_offset + 4
    if coff_header_offset + 8 <= len(data):
        struct.pack_into("<I", data, coff_header_offset + 4, source_date_epoch)

    optional_header_offset = coff_header_offset + 20
    if optional_header_offset + 68 <= len(data):
        struct.pack_into("<I", data, optional_header_offset + 64, 0)

    f.seek(0)
    f.write(data)
    f.truncate()
PY
}

remove_path() {
  local path="$1"

  if [[ ! -e "$path" ]]; then
    return 0
  fi

  for attempt in 1 2 3; do
    if rm -rf "$path"; then
      return 0
    fi

    if [[ "$attempt" == 3 ]]; then
      return 1
    fi

    sleep "$attempt"
  done
}

cleanup() {
  if [[ -n "$MAKENSIS_WRAPPER_DIR" ]]; then
    remove_path "$MAKENSIS_WRAPPER_DIR"
  fi
}

prepare_nsis_makensis_wrapper() {
  if [[ "$BUNDLE_KIND" != "nsis" ]]; then
    return 0
  fi

  MAKENSIS_WRAPPER_DIR="$(mktemp -d)"
  ln -sf "${PWD}/script/ci/makensis-repro-wrapper.sh" "${MAKENSIS_WRAPPER_DIR}/makensis"
  export SONAR_REPO_ROOT="$PWD"
  export PATH="${MAKENSIS_WRAPPER_DIR}:$PATH"
}

run_build() {
  local run="$1"
  local outdir="${OUTPUT_DIR}/${run}"
  local artifact_list="${outdir}/ARTIFACTS"
  local binary_list="${outdir}/BINARIES"
  local search_dirs="${BUNDLE_DIR}"
  local binary_search_dirs="${BINARY_DIR}"

  remove_path dist
  remove_path src-tauri/target
  remove_path "$outdir"
  mkdir -p "$outdir"

  local command=(deno task tauri build --ci --no-sign --verbose)

  if [[ -n "$BUILD_TARGET" ]]; then
    command+=(--target "$BUILD_TARGET")
  fi

  if [[ -n "$BUNDLE_KIND" ]]; then
    command+=(--bundles "$BUNDLE_KIND")
  fi

  if [[ -n "$BUILD_ARGS" ]]; then
    # shellcheck disable=SC2206
    local extra_args=($BUILD_ARGS)
    command+=("${extra_args[@]}")
  fi

  "${command[@]}"

  if [[ -n "$BINARY_PATTERN" ]]; then
    if [[ -n "$BINARY_FALLBACK_DIRS" ]]; then
      binary_search_dirs="${binary_search_dirs}:${BINARY_FALLBACK_DIRS}"
    fi

    : > "$binary_list"
    IFS=':' read -r -a binary_dirs <<< "$binary_search_dirs"
    for dir in "${binary_dirs[@]}"; do
      if [[ -d "$dir" ]]; then
        find "$dir" -maxdepth 1 -type f -name "$BINARY_PATTERN" >> "$binary_list"
      fi
    done
    sort -u "$binary_list" -o "$binary_list"

    if [[ ! -s "$binary_list" ]]; then
      echo "No binary artifact found for pattern $BINARY_PATTERN" >&2
      echo "Searched binary directories:" >&2
      printf '  %s\n' "${binary_dirs[@]}" >&2
      exit 1
    fi

    while IFS= read -r binary; do
      local hash
      hash="$(hash_file "$binary")"
      cp "$binary" "$outdir/"
      printf '%s  %s\n' "$hash" "$(basename "$binary")" >> "$outdir/BINARY_SHA256SUMS"
    done < "$binary_list"
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
    local file_to_hash="$artifact"
    local copied_path="${outdir}/$(basename "$artifact")"

    if [[ "$BUNDLE_KIND" == "nsis" && "$artifact" == *.exe ]]; then
      normalize_nsis_exe_for_hash "$artifact" "$copied_path"
      file_to_hash="$copied_path"
    else
      cp "$artifact" "$copied_path"
    fi

    local hash
    hash="$(hash_file "$file_to_hash")"
    printf '%s  %s\n' "$hash" "$(basename "$artifact")" >> "$outdir/SHA256SUMS"
  done < "$artifact_list"
}

trap cleanup EXIT

ensure_repro_env
prepare_nsis_makensis_wrapper

run_build run1
run_build run2

if [[ -n "$BINARY_PATTERN" ]]; then
  echo "Run 1 binaries:"
  cat "${OUTPUT_DIR}/run1/BINARIES"
  echo "Run 2 binaries:"
  cat "${OUTPUT_DIR}/run2/BINARIES"

  echo "Run 1 binary hashes:"
  cat "${OUTPUT_DIR}/run1/BINARY_SHA256SUMS"
  echo "Run 2 binary hashes:"
  cat "${OUTPUT_DIR}/run2/BINARY_SHA256SUMS"

  if cmp -s "${OUTPUT_DIR}/run1/BINARY_SHA256SUMS" "${OUTPUT_DIR}/run2/BINARY_SHA256SUMS"; then
    echo "Binary is reproducible for ${BUNDLE_LABEL}"
  else
    echo "Binary is not reproducible for ${BUNDLE_LABEL}" >&2
    diff -u "${OUTPUT_DIR}/run1/BINARY_SHA256SUMS" "${OUTPUT_DIR}/run2/BINARY_SHA256SUMS" || true
    exit 1
  fi
fi

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
