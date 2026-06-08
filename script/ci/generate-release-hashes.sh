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

find "$target_dir" -type f \
  \( -name 'sonar' -o -name 'sonar.exe' -o -name 'sonar-*' -o -name 'sonar-*.exe' \) \
  ! -path '*/bundle/*' \
  | sort > binary-artifacts.txt

test -s binary-artifacts.txt

{
  printf '### %s\n\n' "$platform"
  printf '#### Binaire\n\n'

  while IFS= read -r artifact; do
    "${hash_cmd[@]}" "$artifact"
  done < binary-artifacts.txt
} > "$output_file"

grep -Eq '^[0-9a-f]{64} ' "$output_file"
