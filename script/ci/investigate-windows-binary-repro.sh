#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="${OUTPUT_DIR:-windows-binary-repro}"
APP_BINARY_PATTERN="${APP_BINARY_PATTERN:-sonar.exe}"
NSIS_PATTERN="${NSIS_PATTERN:-*.exe}"
EXTRA_RUSTFLAGS="${EXTRA_RUSTFLAGS:-}"

if [[ -n "$EXTRA_RUSTFLAGS" ]]; then
  export RUSTFLAGS="${RUSTFLAGS:-} ${EXTRA_RUSTFLAGS}"
  echo "Using extra RUSTFLAGS: ${EXTRA_RUSTFLAGS}"
  echo "Effective RUSTFLAGS: ${RUSTFLAGS}"
fi

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

record_file() {
  local label="$1"
  local path="$2"
  local outdir="$3"
  local manifest="${outdir}/SHA256SUMS"

  if [[ ! -f "$path" ]]; then
    echo "Missing ${label}: ${path}" >&2
    return 1
  fi

  local hash
  hash="$(hash_file "$path")"
  cp "$path" "${outdir}/${label}"
  printf '%s  %s\n' "$hash" "$label" >> "$manifest"
}

dump_binary_metadata() {
  local label="$1"
  local path="$2"
  local outdir="$3"

  {
    echo "label=${label}"
    echo "path=${path}"
    echo "sha256=$(hash_file "$path")"
    ls -l "$path"

    if command -v rust-readobj >/dev/null 2>&1; then
      echo
      echo "== rust-readobj headers/debug =="
      rust-readobj --file-headers --coff-debug-directory "$path" || true
    fi

    if command -v rust-objdump >/dev/null 2>&1; then
      echo
      echo "== rust-objdump private headers =="
      rust-objdump -p "$path" || true
    fi
  } > "${outdir}/${label}.metadata.txt"

  strings "$path" \
    | grep -Ei '(^[A-Z]:\\|Sonar_desktop_app|runner|target|\.pdb|/workspace|\\workspace)' \
    | sort -u > "${outdir}/${label}.strings.txt" || true
}

find_one() {
  local dir="$1"
  local pattern="$2"
  local label="$3"
  local list
  list="$(find "$dir" -maxdepth 1 -type f -name "$pattern" 2>/dev/null | sort)"

  if [[ -z "$list" ]]; then
    echo "No ${label} found in ${dir} with pattern ${pattern}" >&2
    return 1
  fi

  local count
  count="$(printf '%s\n' "$list" | wc -l | tr -d ' ')"
  if [[ "$count" != "1" ]]; then
    echo "Expected exactly one ${label}, found ${count}:" >&2
    printf '%s\n' "$list" >&2
    return 1
  fi

  printf '%s\n' "$list"
}

run_probe() {
  local run="$1"
  local outdir="${OUTPUT_DIR}/${run}"

  rm -rf dist src-tauri/target "$outdir"
  mkdir -p "$outdir"
  : > "${outdir}/SHA256SUMS"

  echo "== ${run}: build unsigned binary without bundle =="
  deno task tauri build --ci --no-sign --verbose --no-bundle
  local raw_binary
  raw_binary="$(find_one src-tauri/target/release "$APP_BINARY_PATTERN" "raw binary")"
  record_file raw-no-bundle.exe "$raw_binary" "$outdir"
  dump_binary_metadata raw-no-bundle.exe "$raw_binary" "$outdir"

  echo "== ${run}: build NSIS bundle =="
  deno task tauri build --ci --no-sign --verbose --bundles nsis
  local post_bundle_binary
  post_bundle_binary="$(find_one src-tauri/target/release "$APP_BINARY_PATTERN" "post-bundle binary")"
  record_file post-nsis-binary.exe "$post_bundle_binary" "$outdir"
  dump_binary_metadata post-nsis-binary.exe "$post_bundle_binary" "$outdir"

  local nsis_bundle
  nsis_bundle="$(find_one src-tauri/target/release/bundle/nsis "$NSIS_PATTERN" "NSIS bundle")"
  record_file nsis-setup.exe "$nsis_bundle" "$outdir"
  dump_binary_metadata nsis-setup.exe "$nsis_bundle" "$outdir"

  echo "${run} hashes:"
  cat "${outdir}/SHA256SUMS"
}

run_probe run1
run_probe run2

echo "Run 1 hashes:"
cat "${OUTPUT_DIR}/run1/SHA256SUMS"
echo "Run 2 hashes:"
cat "${OUTPUT_DIR}/run2/SHA256SUMS"

if cmp -s "${OUTPUT_DIR}/run1/SHA256SUMS" "${OUTPUT_DIR}/run2/SHA256SUMS"; then
  echo "Windows binary and NSIS outputs are reproducible"
else
  echo "Windows reproducibility investigation found differences" >&2
  diff -u "${OUTPUT_DIR}/run1/SHA256SUMS" "${OUTPUT_DIR}/run2/SHA256SUMS" || true
  for label in raw-no-bundle.exe post-nsis-binary.exe nsis-setup.exe; do
    echo "Metadata diff for ${label}:"
    diff -u "${OUTPUT_DIR}/run1/${label}.metadata.txt" "${OUTPUT_DIR}/run2/${label}.metadata.txt" || true
    echo "String diff for ${label}:"
    diff -u "${OUTPUT_DIR}/run1/${label}.strings.txt" "${OUTPUT_DIR}/run2/${label}.strings.txt" || true
  done
  exit 1
fi
