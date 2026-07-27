#!/usr/bin/env bash
set -euo pipefail

# Démontre la reproductibilité du binaire Linux entre plusieurs builds
# conteneurisés indépendants : N contextes propres (git archive du commit),
# N `docker build` séparés (par défaut sans cache : chaque run reconstruit
# l'environnement complet depuis l'image épinglée par digest et le snapshot
# apt daté), puis comparaison des SHA-256.
#
# Usage :
#   ./script/release/repro-build-container.sh
#
# Variables :
#   RUNS=2       nombre de builds indépendants
#   NO_CACHE=1   1 = --no-cache (docker réellement « différent » à chaque run)
#   OUT=repro-container-out   dossier de sortie (un sous-dossier par run)
#   REF=HEAD     commit à construire
#
# Sortie : $OUT/run<i>/bin/sonar, deb/, rpm/, SHA256SUMS. Code retour 0 si
# tous les binaires sont identiques. Les bundles deb/rpm sont comparés à
# titre informatif (reproductibilité suivie dans #107).

RUNS="${RUNS:-2}"
NO_CACHE="${NO_CACHE:-1}"
OUT="${OUT:-repro-container-out}"
REF="${REF:-HEAD}"
# PLATFORM=linux (binaire + deb/rpm) ou windows (sonar.exe cross-compilé
# via cargo-xwin, cible MSVC).
PLATFORM="${PLATFORM:-linux}"

case "$PLATFORM" in
  linux) target="export"; bin_rel="bin/sonar" ;;
  windows) target="export-windows"; bin_rel="windows/sonar.exe" ;;
  *) echo "PLATFORM invalide : $PLATFORM (linux|windows)" >&2; exit 1 ;;
esac

command -v docker >/dev/null 2>&1 || { echo "docker est requis" >&2; exit 1; }

root="$(git rev-parse --show-toplevel)"
cd "$root"
# OUT accepte un chemin absolu ou relatif au dépôt.
case "$OUT" in
  /*) out_abs="$OUT" ;;
  *) out_abs="$root/$OUT" ;;
esac
commit="$(git rev-parse "$REF")"
epoch="$(git log -1 --format=%ct "$commit")"

echo "Commit construit : $commit (SOURCE_DATE_EPOCH=$epoch)"
echo "Runs : $RUNS — no-cache : $NO_CACHE — plateforme : $PLATFORM"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT
rm -rf "$out_abs"
mkdir -p "$out_abs"

for i in $(seq 1 "$RUNS"); do
  ctx="$workdir/context-run$i"
  mkdir -p "$ctx"
  # Contexte propre par run : uniquement les fichiers versionnés du commit,
  # jamais l'état du working tree.
  git archive "$commit" | tar -x -C "$ctx"

  echo "=== Run $i/$RUNS : docker build ==="
  args=(build --target "$target" --output "type=local,dest=$out_abs/run$i"
    --build-arg "SOURCE_DATE_EPOCH=$epoch")
  if [[ "$NO_CACHE" == "1" ]]; then
    args+=(--no-cache)
  fi
  DOCKER_BUILDKIT=1 docker "${args[@]}" "$ctx"

  (cd "$out_abs/run$i" && find . -type f -exec sha256sum {} + | sort -k2 > ../"run$i.SHA256SUMS" \
    && mv ../"run$i.SHA256SUMS" SHA256SUMS)
done

echo
echo "=== Comparaison ==="
status=0
ref_bin="$(sha256sum "$out_abs/run1/$bin_rel" | cut -d' ' -f1)"
echo "binaire run1 : $ref_bin"
for i in $(seq 2 "$RUNS"); do
  bin="$(sha256sum "$out_abs/run$i/$bin_rel" | cut -d' ' -f1)"
  echo "binaire run$i : $bin"
  if [[ "$bin" != "$ref_bin" ]]; then
    echo "ÉCHEC : le binaire du run$i diffère du run1" >&2
    status=1
  fi
done

# Bundles : informatif uniquement — la reproductibilité deb/rpm est suivie
# dans #107 et ne conditionne pas le code retour.
for i in $(seq 2 "$RUNS"); do
  if diff -q "$out_abs/run1/SHA256SUMS" "$out_abs/run$i/SHA256SUMS" >/dev/null; then
    echo "bundles run$i : identiques au run1 (deb/rpm compris)"
  else
    echo "bundles run$i : différences (attendu tant que #107 est ouvert) :"
    diff "$out_abs/run1/SHA256SUMS" "$out_abs/run$i/SHA256SUMS" | sed 's/^/  /' || true
  fi
done

if [[ "$status" -eq 0 ]]; then
  echo
  echo "OK : $RUNS builds conteneurisés indépendants, binaire identique ($ref_bin)"
fi
exit "$status"
