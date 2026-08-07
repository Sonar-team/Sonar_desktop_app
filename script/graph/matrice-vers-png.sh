#!/usr/bin/env bash
# Rend une matrice de flux (.xlsx ou .csv) en image PNG, avec sigma.js —
# le même moteur de rendu que le graphe de l'application.
#
#   script/graph/matrice-vers-png.sh relevé.xlsx graphe.png
#
# Enchaîne les deux étapes : `sonar-cli graph` produit le snapshot JSON du
# graphe, `render-sigma.mjs` le dessine dans un navigateur sans interface.
#
# Prérequis : un navigateur de la famille Chromium (Brave, Chromium, Chrome…),
# détecté automatiquement — ou imposé par SONAR_BROWSER.
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: matrice-vers-png.sh <matrice.xlsx|matrice.csv> <sortie.png> [options]

Options (transmises au rendu) :
  --width N --height N      Dimensions du rendu (défaut 1600x1000)
  --scale N                 Facteur de résolution (défaut 2)
  --iterations N            Itérations de placement ForceAtlas2 (défaut 600)
  --background <couleur>    Fond de l'image (défaut #ffffff)
  --edge-labels auto|on|off Libellés de protocole sur les liens
  --browser <chemin>        Navigateur à utiliser

Options de sélection (transmises à sonar-cli) :
  --max-nodes N             Nœuds conservés, par trafic décroissant (défaut 200)
  --min-bytes N             Ignore les relations sous ce volume
  --protocol <nom>          Ne garde qu'un protocole (ex. TCP)
  --labels <mode>           Texte des nœuds (ip, mac, label…)
EOF
  exit 2
}

(($# >= 2)) || usage
input="$1"
output="$2"
shift 2

[[ -f "$input" ]] || {
  echo "erreur : $input est introuvable" >&2
  exit 1
}

# Les options se répartissent entre les deux étapes selon leur destinataire.
cli_args=()
render_args=()
while (($# > 0)); do
  case "$1" in
    --max-nodes|--min-bytes|--protocol|--labels)
      cli_args+=("$1" "${2:?valeur manquante pour $1}")
      shift 2
      ;;
    --width|--height|--scale|--iterations|--background|--edge-labels|--browser|--timeout|--seed|--profile)
      render_args+=("$1" "${2:?valeur manquante pour $1}")
      shift 2
      ;;
    -h|--help) usage ;;
    *)
      echo "erreur : option inconnue $1" >&2
      usage
      ;;
  esac
done

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd -- "$here/../.." && pwd)"

cli="$repo/sonar-rust/target/release/sonar-cli"
if [[ ! -x "$cli" ]]; then
  echo "compilation de sonar-cli…" >&2
  (cd "$repo/sonar-rust" && cargo build --release -p sonar-flows-cli -q)
fi

graph_json="$(mktemp --suffix=.json)"
trap 'rm -f "$graph_json"' EXIT

"$cli" graph "$input" -o "$graph_json" "${cli_args[@]}" >&2
node "$here/render-sigma.mjs" --graph "$graph_json" --out "$output" "${render_args[@]}"
