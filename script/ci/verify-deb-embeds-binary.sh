#!/usr/bin/env bash
# Preuve d'inclusion (#136) : le binaire embarqué dans le .deb publié est
# octet pour octet celui qui a passé le smoke test. Sans cette comparaison,
# rien ne relie l'installateur livré au binaire réellement validé.
set -euo pipefail

deb_path="${1:?usage: verify-deb-embeds-binary.sh <paquet.deb> <binaire-testé>}"
binary_path="${2:?usage: verify-deb-embeds-binary.sh <paquet.deb> <binaire-testé>}"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

# Extraction sans dpkg : `ar` + tar suffisent et fonctionnent sur tout runner.
# Chemin absolu résolu AVANT le cd du sous-shell, sinon un chemin relatif se
# résoudrait depuis le répertoire temporaire.
deb_abs="$(cd "$(dirname "$deb_path")" && pwd)/$(basename "$deb_path")"
(cd "$workdir" && ar x "$deb_abs")
data_tar="$(find "$workdir" -maxdepth 1 -name 'data.tar*' | head -n 1)"
if [[ -z "$data_tar" ]]; then
  echo "ERREUR: $deb_path ne contient pas de data.tar*" >&2
  exit 1
fi
mkdir -p "$workdir/root"
tar -xf "$data_tar" -C "$workdir/root"

embedded="$workdir/root/usr/bin/sonar"
if [[ ! -f "$embedded" ]]; then
  echo "ERREUR: $deb_path n'embarque pas usr/bin/sonar" >&2
  find "$workdir/root" -maxdepth 3 -type f >&2
  exit 1
fi

# Unique delta légitime : le marqueur de type de paquet de Tauri
# (`__TAURI_BUNDLE_TYPE_VAR_xxx`), que le bundler patche dans chaque copie
# embarquée (UNK → DEB). On le normalise dans les DEUX fichiers puis on exige
# l'identité stricte : toute autre différence d'octet reste bloquante.
normalized_sha() {
  python3 - "$1" <<'EOF'
import hashlib, re, sys
data = open(sys.argv[1], 'rb').read()
data, count = re.subn(rb'(__TAURI_BUNDLE_TYPE_VAR_)[A-Z]{3}', rb'\1UNK', data)
if count > 8:
    sys.exit(f"marqueur Tauri trouvé {count} fois : inattendu, vérifier le binaire")
print(hashlib.sha256(data).hexdigest())
EOF
}

embedded_sha="$(normalized_sha "$embedded")"
tested_sha="$(normalized_sha "$binary_path")"

if [[ "$embedded_sha" != "$tested_sha" ]]; then
  echo "ERREUR: le binaire embarqué dans $(basename "$deb_path") diffère du binaire testé" >&2
  echo "  embarqué (normalisé) : $embedded_sha" >&2
  echo "  testé    (normalisé) : $tested_sha" >&2
  echo "Le .deb ne doit PAS être publié : il contient des octets jamais validés." >&2
  exit 1
fi

echo "OK: $(basename "$deb_path") embarque le binaire testé, au marqueur de type"
echo "de paquet Tauri près (hash normalisé $tested_sha)"
