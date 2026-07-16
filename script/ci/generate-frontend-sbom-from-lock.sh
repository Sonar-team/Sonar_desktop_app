#!/usr/bin/env bash
# Génère le SBOM CycloneDX du frontend depuis deno.lock (#137).
#
# Syft ne cataloguait pas deno.lock et node_modules était exclu du scan : le
# SBOM frontend ne contenait aucune dépendance JS. Le lockfile est la source
# de vérité exacte (npm résolus, transitifs compris, avec intégrité) ; on le
# convertit directement, sans dépendre du support Deno d'un outil tiers.
# Sortie volontairement sans timestamp ni serialNumber : le SBOM est
# reproductible à contenu de lockfile égal.
#
# Usage : generate-frontend-sbom-from-lock.sh <deno.lock> <version> <sortie.cdx.json>
set -euo pipefail

LOCKFILE="${1:?chemin de deno.lock requis}"
VERSION="${2:?version requise}"
OUTPUT="${3:?fichier de sortie requis}"

lock_version="$(jq -r '.version' "$LOCKFILE")"
if [[ "$lock_version" != "5" ]]; then
  echo "deno.lock version $lock_version inattendue (script écrit pour la v5) :" >&2
  echo "vérifier que .npm est toujours un objet « name@version » -> { integrity }" >&2
  exit 1
fi

jq --arg version "$VERSION" '
{
  bomFormat: "CycloneDX",
  specVersion: "1.5",
  version: 1,
  metadata: {
    component: {
      type: "application",
      name: "sonar-frontend",
      version: $version
    }
  },
  components: (.npm // {} | to_entries | map(
    # Clé deno.lock : « name@version », parfois suivie du suffixe de
    # résolution des peer-deps « _pkg@ver__… » à écarter. Le nom peut être
    # scopé (@scope/name) ; la version ne contient jamais de « _ ».
    (.key | capture("^(?<name>@?[^@]+(?:/[^@]+)?)@(?<ver>[^_]+)")) as $nv |
    {
      type: "library",
      name: $nv.name,
      version: $nv.ver,
      purl: ("pkg:npm/" + ($nv.name | sub("^@"; "%40")) + "@" + $nv.ver)
    }
    + (if .value.integrity then
        { properties: [ { name: "sonar:lock:integrity", value: .value.integrity } ] }
      else {} end)
  ) | sort_by(.purl))
}' "$LOCKFILE" > "$OUTPUT"

count="$(jq '.components | length' "$OUTPUT")"
if [[ "$count" -eq 0 ]]; then
  echo "SBOM frontend vide : aucune dépendance npm trouvée dans $LOCKFILE" >&2
  exit 1
fi
echo "SBOM frontend : $count composant(s) npm depuis $LOCKFILE -> $OUTPUT"
