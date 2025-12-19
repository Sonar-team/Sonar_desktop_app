#!/usr/bin/env bash
set -euo pipefail

OWNER="Sonar-team"
REPO="Sonar_desktop_app"
TAG_PREFIX="app-v"

# Optionnel: token GitHub pour éviter le rate limit (recommandé).
#   export GITHUB_TOKEN="ghp_xxx"
AUTH_HEADER=()
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
  AUTH_HEADER=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
fi

API_BASE="https://api.github.com/repos/${OWNER}/${REPO}"

# Classement simple par OS, basé sur l'extension
classify_os() {
  local name="$1"
  shopt -s nocasematch
  if [[ "$name" =~ \.msi$|\.exe$ ]]; then
    echo "windows"
  elif [[ "$name" =~ \.deb$|\.rpm$|\.appimage$ ]]; then
    echo "linux"
  elif [[ "$name" =~ \.dmg$ ]]; then
    echo "macos"
  elif [[ "$name" =~ \.tar\.gz$|\.zip$ ]]; then
    echo "source_or_archive"
  else
    echo "other"
  fi
  shopt -u nocasematch
}

# Récupère toutes les releases (pagination)
fetch_all_releases_json() {
  local page=1
  while :; do
    local resp
    resp="$(curl -sS "${AUTH_HEADER[@]}" \
      "${API_BASE}/releases?per_page=100&page=${page}")"

    local count
    count="$(echo "$resp" | jq 'length')"
    if [[ "$count" -eq 0 ]]; then
      break
    fi

    echo "$resp"
    page=$((page + 1))
  done
}

# -----------------------
# Main
# -----------------------

TMP_JSON="$(mktemp)"
trap 'rm -f "$TMP_JSON"' EXIT

# Concatène les pages en un seul tableau JSON
# Chaque page est un tableau ; on "slurp" puis on aplati.
fetch_all_releases_json | jq -s 'add' > "$TMP_JSON"

# Filtre releases taggées app-v*
# NB: Certaines releases peuvent être "draft" ou "prerelease" ; on les garde quand même, mais tu peux filtrer.
mapfile -t TAGS < <(jq -r --arg pfx "$TAG_PREFIX" '
  .[]
  | select(.tag_name | startswith($pfx))
  | .tag_name
' "$TMP_JSON" | sort -V)

if [[ "${#TAGS[@]}" -eq 0 ]]; then
  echo "Aucune release trouvée avec le préfixe '${TAG_PREFIX}' sur ${OWNER}/${REPO}."
  exit 1
fi

CSV_OUT="github_release_downloads_${OWNER}_${REPO}.csv"
echo "tag,published_at,total,windows,linux,macos,source_or_archive,other" > "$CSV_OUT"

echo "Repo: ${OWNER}/${REPO}"
echo "Releases détectées (${#TAGS[@]}): ${TAGS[*]}"
echo

grand_total=0
printf "%-12s  %-20s  %8s  %8s  %8s  %8s  %16s  %8s\n" \
  "TAG" "PUBLISHED_AT" "TOTAL" "WIN" "LINUX" "MAC" "SRC/ARCHIVE" "OTHER"
printf "%s\n" "-----------------------------------------------------------------------------------------------"

for tag in "${TAGS[@]}"; do
  rel_json="$(curl -sS "${AUTH_HEADER[@]}" "${API_BASE}/releases/tags/${tag}")"

  published_at="$(echo "$rel_json" | jq -r '.published_at // .created_at // ""')"

  # Liste des assets (name + download_count)
  # Si pas d'assets, totals = 0
  mapfile -t assets < <(echo "$rel_json" | jq -r '.assets[]? | "\(.name)\t\(.download_count)"')

  total=0
  win=0
  linux=0
  mac=0
  src=0
  other=0

  for line in "${assets[@]:-}"; do
    name="${line%%$'\t'*}"
    count="${line##*$'\t'}"

    # Sécurité : si vide
    count="${count:-0}"

    os="$(classify_os "$name")"
    total=$((total + count))

    case "$os" in
      windows) win=$((win + count)) ;;
      linux) linux=$((linux + count)) ;;
      macos) mac=$((mac + count)) ;;
      source_or_archive) src=$((src + count)) ;;
      *) other=$((other + count)) ;;
    esac
  done

  grand_total=$((grand_total + total))

  printf "%-12s  %-20s  %8d  %8d  %8d  %8d  %16d  %8d\n" \
    "$tag" "$published_at" "$total" "$win" "$linux" "$mac" "$src" "$other"

  echo "${tag},${published_at},${total},${win},${linux},${mac},${src},${other}" >> "$CSV_OUT"
done

printf "%s\n" "-----------------------------------------------------------------------------------------------"
echo "TOTAL CUMULÉ (toutes releases ${TAG_PREFIX}*): ${grand_total}"
echo
echo "CSV généré: ${CSV_OUT}"
echo "Astuce: ouvre-le avec LibreOffice/Excel ou commit-le dans le repo pour un snapshot."
