#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: update-release-body.sh <release-tag> [hash-dir] [body-file]}"
hash_dir="${2:-release-hashes}"
body_file="${3:-release-body.md}"

{
  printf 'Cette release publie les binaires construits avec l'\''environnement reproductible et les bundles natifs generes par Tauri: DMG pour macOS, DEB/RPM pour Linux, NSIS pour Windows.\n\n'
  printf 'La verification de reproductibilite porte sur le binaire; les bundles sont publies avec hashes, attestations et signatures detachees.\n\n'
  printf 'Chaque plateforme fournit aussi un kit hors ligne signe contenant les artefacts, leurs preuves Sigstore, la racine de confiance et le verificateur versionne.\n\n'
  printf '## Windows\n\n'
  printf 'Npcap n'\''est pas inclus dans SONAR. Le bundle NSIS détecte son absence ou un mode incompatible et propose d'\''ouvrir https://npcap.com/#download. Installez Npcap séparément avec le mode compatible WinPcap avant de lancer SONAR.\n\n'
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
