#!/usr/bin/env bash
# Upload d'assets vers une release, sans jamais remplacer (#136).
#
# `gh release upload --clobber` écrase silencieusement un asset existant :
# sur un rerun, les octets publiés peuvent changer sous les pieds de qui a
# déjà téléchargé et vérifié un hash. Ce script refuse le remplacement.
#
# Idempotence : un asset déjà présent avec le MÊME contenu est accepté (le
# rerun d'un job interrompu ne doit pas bloquer la release) ; un asset
# présent avec un contenu DIFFÉRENT est une erreur fatale.
set -euo pipefail

tag="${1:?usage: upload-release-assets-once.sh <tag> <fichier...>}"
shift

if (($# == 0)); then
  echo "ERREUR: aucun fichier à publier pour $tag" >&2
  exit 1
fi

# Assets déjà attachés, et leur taille — seule métadonnée que l'API expose
# sans télécharger. Un asset de taille différente est refusé immédiatement ;
# à taille égale, on télécharge pour comparer les octets.
existing_json="$(gh release view "$tag" --json assets --jq '.assets')"

for file in "$@"; do
  [[ -f "$file" ]] || {
    echo "ERREUR: $file n'est pas un fichier" >&2
    exit 1
  }
  name="$(basename "$file")"
  local_size="$(wc -c <"$file" | tr -d ' ')"
  remote_size="$(jq -r --arg n "$name" \
    '(.[] | select(.name == $n) | .size) // empty' <<<"$existing_json")"

  if [[ -z "$remote_size" ]]; then
    echo "upload: $name"
    gh release upload "$tag" "$file"
    continue
  fi

  if [[ "$remote_size" != "$local_size" ]]; then
    echo "ERREUR: $name existe déjà sur $tag avec une taille différente" >&2
    echo "  publié : ${remote_size} octets" >&2
    echo "  local  : ${local_size} octets" >&2
    echo "Un asset publié ne doit jamais être remplacé : retirez-le" >&2
    echo "explicitement ou publiez un nouveau tag." >&2
    exit 1
  fi

  # Taille identique : comparaison du contenu réel avant de conclure.
  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064 # expansion voulue à la définition du trap
  trap "rm -rf '$tmp'" RETURN
  gh release download "$tag" --pattern "$name" --dir "$tmp" --clobber
  if cmp -s "$file" "$tmp/$name"; then
    echo "déjà publié à l'identique, ignoré: $name"
  else
    echo "ERREUR: $name existe déjà sur $tag avec un contenu différent" >&2
    echo "  (même taille, octets divergents — build non reproductible ou" >&2
    echo "  artefact substitué). Publication refusée." >&2
    exit 1
  fi
  rm -rf "$tmp"
  trap - RETURN
done
