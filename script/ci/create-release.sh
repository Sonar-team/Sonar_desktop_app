#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?usage: create-release.sh <release-tag> [body-file]}"
body_file="${2:-release-body.md}"

{
  printf 'Cette release publie les binaires construits avec l'\''environnement reproductible et les bundles natifs generes par Tauri: DMG pour macOS, DEB/RPM pour Linux, NSIS pour Windows.\n\n'
  printf 'La verification de reproductibilite porte sur le binaire; les bundles sont publies avec hashes, attestations et signatures detachees.\n\n'
  printf 'Chaque plateforme fournit aussi un kit hors ligne signe contenant les artefacts, leurs preuves Sigstore, la racine de confiance et le verificateur versionne.\n\n'
  printf '## Windows\n\n'
  printf 'Npcap n'\''est pas inclus dans SONAR. Le bundle NSIS détecte son absence ou un mode incompatible et propose d'\''ouvrir https://npcap.com/#download. Installez Npcap séparément avec le mode compatible WinPcap avant de lancer SONAR.\n'
} > "$body_file"

test -s "$body_file"

# Release atomique (#136) : créée en draft, rendue publique par le job
# finalize-release seulement quand builds, smoke tests, hashes et
# attestations ont réussi sur toutes les plateformes. Un tag dont un build
# échoue ne laisse donc aucune release publique visible.
if gh release view "$release_tag" >/dev/null 2>&1; then
  gh release edit "$release_tag" \
    --title "Sonar ${release_tag}" \
    --notes-file "$body_file"
else
  gh release create "$release_tag" \
    --title "Sonar ${release_tag}" \
    --notes-file "$body_file" \
    --draft \
    --verify-tag
fi
