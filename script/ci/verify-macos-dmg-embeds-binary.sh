#!/usr/bin/env bash
# Preuve d'inclusion macOS (#136) : le binaire dans le DMG produit est
# exactement celui qui a passé le smoke test.
#
# Contrairement au .deb (dont la vérification doit normaliser le marqueur de
# type de paquet que Tauri patche), le DMG ne fait que transporter le .app :
# l'identité doit être stricte, octet pour octet.
set -euo pipefail

dmg_path="${1:?usage: verify-macos-dmg-embeds-binary.sh <image.dmg> <binaire-testé>}"
binary_path="${2:?usage: verify-macos-dmg-embeds-binary.sh <image.dmg> <binaire-testé>}"

mount_point="$(mktemp -d)"
attached=0
cleanup() {
  if ((attached == 1)); then
    hdiutil detach "$mount_point" -quiet || true
  fi
  rm -rf "$mount_point"
}
trap cleanup EXIT

hdiutil attach "$dmg_path" -mountpoint "$mount_point" -nobrowse -readonly -quiet
attached=1

embedded="$(find "$mount_point" -type f -path '*.app/Contents/MacOS/*' -print -quit)"
if [[ -z "$embedded" ]]; then
  echo "ERREUR: aucun exécutable trouvé dans le .app du DMG" >&2
  find "$mount_point" -maxdepth 3 >&2
  exit 1
fi

embedded_sha="$(shasum -a 256 "$embedded" | cut -d' ' -f1)"
tested_sha="$(shasum -a 256 "$binary_path" | cut -d' ' -f1)"

if [[ "$embedded_sha" != "$tested_sha" ]]; then
  echo "ERREUR: le binaire du DMG diffère du binaire testé" >&2
  echo "  embarqué : $embedded_sha" >&2
  echo "  testé    : $tested_sha" >&2
  echo "Le DMG ne doit PAS être publié : il contient des octets jamais validés." >&2
  exit 1
fi

echo "OK: $(basename "$dmg_path") embarque exactement le binaire testé ($tested_sha)"
