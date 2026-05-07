#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: generate-release-hashes.sh <platform> [target-dir] [output-file]}"
target_dir="${2:-src-tauri/target}"
output_file="${3:-release-hashes-${platform}.md}"

if command -v sha256sum >/dev/null 2>&1; then
  hash_cmd=(sha256sum)
else
  hash_cmd=(shasum -a 256)
fi

find "$target_dir" -type f \
  \( -path '*/release/sonar' -o -path '*/release/sonar.exe' \) \
  ! -path '*/bundle/*' \
  | sort > binary-artifacts.txt

find "$target_dir" -type f \
  \( -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' -o -name '*.dmg' -o -name '*.msi' -o -name '*.exe' \) \
  -path '*/bundle/*' \
  | sort > bundle-artifacts.txt

test -s binary-artifacts.txt
test -s bundle-artifacts.txt

{
  printf '### %s\n\n' "$platform"
  printf '#### Binaire\n\n'

  while IFS= read -r artifact; do
    "${hash_cmd[@]}" "$artifact"
  done < binary-artifacts.txt

  printf '\n#### Bundles\n\n'

  while IFS= read -r artifact; do
    "${hash_cmd[@]}" "$artifact"
  done < bundle-artifacts.txt
} > "$output_file"

grep -Eq '^[0-9a-f]{64} ' "$output_file"
