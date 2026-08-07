#!/usr/bin/env bash
# Test du garde-fou d'idempotence de la publication (#136), avec un faux
# `gh` : prouve qu'une divergence d'artefact bloque AVANT publication, sans
# toucher à GitHub. C'est le « test de workflow » exigé par les critères
# d'acceptation de l'issue.
#
# Lancé par le job `release-guard-tests` de rust-ci.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$(cd "$root/.." && pwd)/upload-release-assets-once.sh"
work="$(mktemp -d)"
rm -rf "$work"
mkdir -p "$work/bin" "$work/files" "$work/remote"

# Faux `gh` : `release view` rend les assets du dossier remote,
# `release download` les copie, `release upload` les ajoute.
cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
remote="$FAKE_REMOTE"
case "$2" in
  view)
    printf '['
    first=1
    for f in "$remote"/*; do
      [ -e "$f" ] || continue
      [ $first -eq 1 ] || printf ','
      first=0
      printf '{"name":"%s","size":%s}' "$(basename "$f")" "$(wc -c <"$f" | tr -d ' ')"
    done
    printf ']\n'
    ;;
  download)
    for ((i = 1; i <= $#; i++)); do
      if [ "${!i}" = "--pattern" ]; then j=$((i + 1)); pattern="${!j}"; fi
      if [ "${!i}" = "--dir" ]; then j=$((i + 1)); dir="${!j}"; fi
    done
    cp "$remote/$pattern" "$dir/$pattern"
    ;;
  upload)
    echo "UPLOADED $4" >>"$FAKE_LOG"
    cp "$4" "$remote/$(basename "$4")"
    ;;
esac
FAKE
chmod +x "$work/bin/gh"

export PATH="$work/bin:$PATH"
export FAKE_REMOTE="$work/remote"
export FAKE_LOG="$work/log"
: >"$FAKE_LOG"

fail=0
check() {
  if [ "$1" = "$2" ]; then
    echo "  OK   — $3"
  else
    echo "  ÉCHEC — $3 (attendu '$1', obtenu '$2')"
    fail=1
  fi
}

printf 'contenu-a' >"$work/files/asset.bin"

echo "1. asset absent : upload"
set +e
"$script" v1.0.0 "$work/files/asset.bin" >/dev/null 2>&1
check 0 $? "sortie 0"
set -e
check "UPLOADED $work/files/asset.bin" "$(cat "$FAKE_LOG")" "upload effectué"

echo "2. asset identique : ignoré, pas de second upload"
: >"$FAKE_LOG"
set +e
"$script" v1.0.0 "$work/files/asset.bin" >/dev/null 2>&1
check 0 $? "sortie 0 (idempotent)"
set -e
check "" "$(cat "$FAKE_LOG")" "aucun upload"

echo "3. même taille, contenu différent : REFUSÉ"
printf 'contenu-b' >"$work/files/asset.bin"
set +e
out="$("$script" v1.0.0 "$work/files/asset.bin" 2>&1)"
code=$?
set -e
check 1 $code "sortie 1"
case "$out" in *"contenu différent"*) echo "  OK   — message explicite" ;; *) echo "  ÉCHEC — message"; fail=1 ;; esac

echo "4. taille différente : REFUSÉ"
printf 'contenu-beaucoup-plus-long' >"$work/files/asset.bin"
set +e
out="$("$script" v1.0.0 "$work/files/asset.bin" 2>&1)"
code=$?
set -e
check 1 $code "sortie 1"
case "$out" in *"taille différente"*) echo "  OK   — message explicite" ;; *) echo "  ÉCHEC — message"; fail=1 ;; esac

rm -rf "$work"
if [ $fail -eq 0 ]; then echo "TOUS LES CAS PASSENT"; else echo "DES CAS ÉCHOUENT"; exit 1; fi
