#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: create-release.sh <release-tag> [body-file]}"
body_file="${2:-release-body.md}"

{
  printf 'Cette release publie les binaires construits avec l'\''environnement reproductible et les bundles natifs generes par Tauri: DMG pour macOS, DEB/RPM pour Linux, MSI/NSIS pour Windows.\n\n'
  printf 'La verification de reproductibilite porte sur le binaire; les bundles sont publies avec hashes, attestations et signatures detachees.\n\n'
  printf '## Windows\n\n'
  printf 'Le bundle NSIS peut proposer l'\''installation de Npcap. Pour le binaire brut `sonar.exe` ou le MSI, installez Npcap séparément depuis https://npcap.com/#download et activez le mode compatible WinPcap dans l'\''installateur Npcap. Sans Npcap, la capture réseau ne fonctionnera pas.\n'
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
