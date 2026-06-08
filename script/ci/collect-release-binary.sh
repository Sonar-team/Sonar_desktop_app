#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: collect-release-binary.sh <platform> [target-dir] [output-dir]}"
target_dir="${2:-src-tauri/target}"
output_dir="${3:-release-artifacts}"

case "$platform" in
  macos-14) artifact_name="sonar-macos-aarch64" ;;
  ubuntu-22.04) artifact_name="sonar-linux-x86_64" ;;
  windows-2022) artifact_name="sonar-windows-x86_64.exe" ;;
  *)
    safe_platform="$(printf '%s' "$platform" | tr -cs 'A-Za-z0-9._-' '-')"
    artifact_name="sonar-${safe_platform}"
    ;;
esac

if [[ "$platform" == windows-* && "$artifact_name" != *.exe ]]; then
  artifact_name="${artifact_name}.exe"
fi

binary_list="$(mktemp)"
trap 'rm -f "$binary_list"' EXIT

find "$target_dir" -type f \
  \( -path '*/release/sonar' -o -path '*/release/sonar.exe' \) \
  ! -path '*/bundle/*' \
  | sort > "$binary_list"

binary_count="$(wc -l < "$binary_list" | tr -d '[:space:]')"

if [[ "$binary_count" -eq 0 ]]; then
  find "$target_dir" -type f \
    \( -path '*/release/deps/sonar-*' -o -path '*/release/deps/sonar-*.exe' \) \
    ! -name '*.d' \
    ! -name '*.rlib' \
    ! -name '*.so' \
    ! -path '*/bundle/*' \
    -perm /111 \
    | sort > "$binary_list"
  binary_count="$(wc -l < "$binary_list" | tr -d '[:space:]')"
fi

if [[ "$binary_count" -ne 1 ]]; then
  echo "Expected exactly one release binary, found ${binary_count}:" >&2
  sed 's/^/  /' "$binary_list" >&2
  exit 1
fi

binary_path="$(cat "$binary_list")"

mkdir -p "$output_dir"
cp "$binary_path" "${output_dir}/${artifact_name}"
printf '%s\n' "${output_dir}/${artifact_name}"
