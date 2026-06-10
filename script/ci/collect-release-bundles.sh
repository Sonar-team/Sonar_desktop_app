#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: collect-release-bundles.sh <platform> [target-dir] [output-dir]}"
target_dir="${2:-src-tauri/target}"
output_dir="${3:-release-artifacts}"

case "$platform" in
  macos-14)
    artifact_prefix="sonar-macos-aarch64"
    patterns=("*.dmg")
    suffixes=("dmg")
    ;;
  ubuntu-22.04)
    artifact_prefix="sonar-linux-x86_64"
    patterns=("*.deb" "*.rpm")
    suffixes=("deb" "rpm")
    ;;
  windows-2022)
    artifact_prefix="sonar-windows-x86_64"
    patterns=("*.msi" "*setup.exe")
    suffixes=("msi" "setup.exe")
    ;;
  *)
    safe_platform="$(printf '%s' "$platform" | tr -cs 'A-Za-z0-9._-' '-')"
    artifact_prefix="sonar-${safe_platform}"
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

  output_path="${output_dir}/${artifact_prefix}.${suffix}"
  if [[ "$suffix" == "setup.exe" ]]; then
    output_path="${output_dir}/${artifact_prefix}-setup.exe"
  fi

  cp "$bundle_path" "$output_path"
  printf '%s\n' "$output_path"
  bundle_count=$((bundle_count + 1))
done

if [[ "$bundle_count" -eq 0 ]]; then
  echo "No release bundles found for ${platform} under ${target_dir}" >&2
  exit 1
fi
