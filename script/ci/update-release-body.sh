#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: update-release-body.sh <release-tag> [hash-dir] [body-file]}"
hash_dir="${2:-release-hashes}"
body_file="${3:-release-body.md}"

{
  printf 'Voir les assets ci-dessous pour télécharger cette version.\n\n'
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
