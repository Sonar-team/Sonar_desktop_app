#!/usr/bin/env bash
# Empaquette un .app déjà construit en DMG (#136).
#
# Pourquoi ce script plutôt que le bundler DMG de Tauri : celui-ci échoue
# lorsque le build tourne dans l'environnement reproductible
# (`security/repro-env.ts`, qui pose SOURCE_DATE_EPOCH) — constaté sur
# macos-14, la sortie d'erreur de son `bundle_dmg.sh` étant avalée.
#
# La propriété qui compte pour #136 est préservée, et même renforcée : le
# .app est produit par LE build unique qui a fourni le binaire smoke-testé,
# et il est ici seulement *recopié* dans une image disque. Aucun cargo n'est
# réinvoqué, donc aucun octet ne peut différer de ce qui a été testé.
#
# Contrepartie assumée : pas de fond ni de mise en page d'icônes
# personnalisés (ce que faisait bundle_dmg.sh). Le DMG reste un volume
# contenant l'application et un lien vers /Applications.
set -euo pipefail

app_path="${1:?usage: package-macos-dmg.sh <chemin.app> <sortie.dmg>}"
dmg_path="${2:?usage: package-macos-dmg.sh <chemin.app> <sortie.dmg>}"

if [[ ! -d "$app_path" ]]; then
  echo "ERREUR: $app_path est introuvable ou n'est pas un bundle .app" >&2
  exit 1
fi

staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT

# Le volume ne contient que l'application et le raccourci d'installation
# habituel — glisser-déposer, sans mise en page scriptée.
cp -R "$app_path" "$staging/"
ln -s /Applications "$staging/Applications"

volume_name="$(basename "$app_path" .app)"
mkdir -p "$(dirname "$dmg_path")"
rm -f "$dmg_path"

# UDZO : image compressée en lecture seule, le format usuel de distribution.
hdiutil create \
  -volname "$volume_name" \
  -srcfolder "$staging" \
  -ov \
  -format UDZO \
  "$dmg_path"

echo "DMG créé : $dmg_path"
