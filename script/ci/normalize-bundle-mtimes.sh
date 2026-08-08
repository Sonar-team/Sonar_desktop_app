#!/usr/bin/env bash
# Normalise les dates de modification de tout ce que le bundler va empaqueter
# (#119, #107, #120).
#
# Pourquoi : les formats d'installateur enregistrent la mtime de chaque
# fichier empaqueté. Le binaire de SONAR est reproductible bit à bit, mais
# chaque compilation le réécrit, donc sa mtime change — et l'installateur
# diffère alors que son contenu est identique. Vérifié expérimentalement sur
# NSIS le 07/08/2026 : contenu identique + mtime différente = 669 octets de
# divergence ; mtimes normalisées = octets identiques.
#
# Ce script est appelé par `beforeBundleCommand` de tauri.conf.json : il
# s'exécute APRÈS la compilation et AVANT l'empaquetage, dans la même
# invocation de `tauri build` — la garantie de #136 (un seul build) est
# préservée, aucun cargo n'est réinvoqué.
#
# SOURCE_DATE_EPOCH vient de `security/repro-env.ts` en release. Hors de cet
# environnement (build de développement), le script ne fait rien : rien à
# normaliser si la reproductibilité n'est pas demandée.
set -euo pipefail

if [[ -z "${SOURCE_DATE_EPOCH:-}" ]]; then
  echo "[normalize-mtimes] SOURCE_DATE_EPOCH absent : normalisation ignorée"
  exit 0
fi

if [[ ! "$SOURCE_DATE_EPOCH" =~ ^[0-9]+$ ]]; then
  echo "[normalize-mtimes] SOURCE_DATE_EPOCH invalide : $SOURCE_DATE_EPOCH" >&2
  exit 1
fi

root_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
target_dir="${CARGO_TARGET_DIR:-$root_dir/src-tauri/target}"

# Les artefacts déjà empaquetés (bundle/) sont exclus : les normaliser ne
# servirait à rien et masquerait une divergence réelle du contenu.
normalized=0
while IFS= read -r -d '' path; do
  touch -h -d "@${SOURCE_DATE_EPOCH}" "$path" 2>/dev/null && normalized=$((normalized + 1))
done < <(
  find "$target_dir" \
    -mindepth 1 \
    -path '*/bundle' -prune -o \
    \( -type f -o -type l \) -print0 2>/dev/null
)

# Les ressources embarquées viennent aussi du dépôt (icônes, images NSIS,
# frontend compilé) : leur mtime part dans l'installateur au même titre.
for extra in "$root_dir/dist" "$root_dir/src-tauri/icons" "$root_dir/src-tauri/windows"; do
  [[ -d "$extra" ]] || continue
  while IFS= read -r -d '' path; do
    touch -h -d "@${SOURCE_DATE_EPOCH}" "$path" 2>/dev/null && normalized=$((normalized + 1))
  done < <(find "$extra" \( -type f -o -type l \) -print0 2>/dev/null)
done

echo "[normalize-mtimes] ${normalized} fichier(s) datés à ${SOURCE_DATE_EPOCH}"
