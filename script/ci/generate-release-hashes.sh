#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: generate-release-hashes.sh <platform> [artifact-dir] [output-file]}"
target_dir="${2:-release-artifacts}"
output_file="${3:-release-hashes-${platform}.md}"

if command -v sha256sum >/dev/null 2>&1; then
  hash_cmd=(sha256sum)
else
  hash_cmd=(shasum -a 256)
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

find "$target_dir" -type f | sort > "${tmpdir}/release-artifacts.txt"

test -s "${tmpdir}/release-artifacts.txt"

{
  printf '### %s\n\n' "$platform"
  printf '#### Artifacts\n\n'

  while IFS= read -r artifact; do
    "${hash_cmd[@]}" "$artifact"
  done < "${tmpdir}/release-artifacts.txt"
} > "$output_file"

grep -Eq '^[0-9a-f]{64} ' "$output_file"
