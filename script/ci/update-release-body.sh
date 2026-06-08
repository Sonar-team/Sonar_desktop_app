#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: update-release-body.sh <release-tag> [hash-dir] [body-file]}"
hash_dir="${2:-release-hashes}"
body_file="${3:-release-body.md}"

{
  printf 'Cette release publie les binaires reproductibles, pas des installateurs.\n\n'
  printf '## Windows\n\n'
  printf 'Avant de lancer `sonar.exe`, installez Npcap séparément depuis https://npcap.com/#download et activez le mode compatible WinPcap dans l'\''installateur Npcap. Sans Npcap, la capture réseau ne fonctionnera pas.\n\n'
  printf '## SHA256\n\n'
  find "$hash_dir" -type f -name 'release-hashes-*.md' \
    | sort | while IFS= read -r hash_file; do
      cat "$hash_file"
      printf '\n'
    done
} > "$body_file"

test -s "$body_file"
grep -q '^## SHA256$' "$body_file"
grep -Eq '^[0-9a-f]{64} ' "$body_file"

gh release edit "$release_tag" --notes-file "$body_file"
