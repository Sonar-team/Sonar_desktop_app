#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: create-release.sh <release-tag> [body-file]}"
body_file="${2:-release-body.md}"

{
  printf 'Cette release publie les binaires reproductibles, pas des installateurs.\n\n'
  printf '## Windows\n\n'
  printf 'Avant de lancer `sonar.exe`, installez Npcap séparément depuis https://npcap.com/#download et activez le mode compatible WinPcap dans l'\''installateur Npcap. Sans Npcap, la capture réseau ne fonctionnera pas.\n'
} > "$body_file"

test -s "$body_file"

if gh release view "$release_tag" >/dev/null 2>&1; then
  gh release edit "$release_tag" \
    --title "Sonar ${release_tag}" \
    --notes-file "$body_file"
else
  gh release create "$release_tag" \
    --title "Sonar ${release_tag}" \
    --notes-file "$body_file" \
    --verify-tag
fi
