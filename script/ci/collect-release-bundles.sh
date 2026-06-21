#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: collect-release-bundles.sh <platform> [target-dir] [output-dir]}"
target_dir="${2:-src-tauri/target}"
output_dir="${3:-release-artifacts}"

case "$platform" in
  macos-14)
    patterns=("*.dmg")
    suffixes=("dmg")
    ;;
  ubuntu-22.04)
    patterns=("*.deb" "*.rpm")
    suffixes=("deb" "rpm")
    ;;
  windows-2022)
    patterns=("*.msi" "*setup.exe")
    suffixes=("msi" "setup.exe")
    ;;
  *)
    patterns=("*.dmg" "*.deb" "*.rpm" "*.msi" "*setup.exe")
    suffixes=("dmg" "deb" "rpm" "msi" "setup.exe")
    ;;
esac

mkdir -p "$output_dir"

bundle_count=0
for index in "${!patterns[@]}"; do
  pattern="${patterns[$index]}"
  suffix="${suffixes[$index]}"
  bundle_list="$(mktemp)"

  find "$target_dir" -type f \
    -path '*/bundle/*' \
    -name "$pattern" \
    | sort > "$bundle_list"

  count="$(wc -l < "$bundle_list" | tr -d '[:space:]')"
  if [[ "$count" -eq 0 ]]; then
    rm -f "$bundle_list"
    continue
  fi

  if [[ "$count" -ne 1 ]]; then
    echo "Expected exactly one ${suffix} bundle for ${platform}, found ${count}:" >&2
    sed 's/^/  /' "$bundle_list" >&2
    rm -f "$bundle_list"
    exit 1
  fi

  bundle_path="$(cat "$bundle_list")"
  rm -f "$bundle_list"

  output_path="${output_dir}/$(basename "$bundle_path")"
  cp "$bundle_path" "$output_path"
  printf '%s\n' "$output_path"
  bundle_count=$((bundle_count + 1))
done

if [[ "$bundle_count" -eq 0 ]]; then
  echo "No release bundles found for ${platform} under ${target_dir}" >&2
  exit 1
fi
