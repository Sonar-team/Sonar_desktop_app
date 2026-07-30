#!/usr/bin/env bash
set -euo pipefail

SOURCE_DEB="${1:-}"
OUTPUT_DEB="${2:-}"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-1700000000}"

usage() {
  cat <<'EOF'
usage: script/package-deb-repro.sh [source.deb] [output.deb]

Repack a Debian package into a normalized, reproducible .deb.
If no source package is provided, the first bundle/deb package is used.
If no output path is provided, a normalized copy is written to dist/repro-deb/.
EOF
}

log() {
  printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing command: $1" >&2
    exit 1
  }
}

resolve_source_deb() {
  if [[ -n "$SOURCE_DEB" ]]; then
    printf '%s\n' "$SOURCE_DEB"
    return 0
  fi

  find src-tauri/target/release/bundle/deb -maxdepth 2 -type f -name '*.deb' 2>/dev/null | sort | head -n 1
}

make_list() {
  local base="$1"
  local out="$2"

  (
    cd "$base"
    find . -mindepth 1 -print0 | sort -z | sed -z 's#^\./##'
  ) > "$out"
}

normalize_package_tree() {
  local root="$1"

  find "$root" -exec touch -h -d "@${SOURCE_DATE_EPOCH}" {} +
}

make_tar_xz() {
  local root="$1"
  local list_file="$2"
  local output_file="$3"

  if [[ ! -s "$list_file" ]]; then
    echo "Empty archive input list: $list_file" >&2
    exit 1
  fi

  # --no-recursion : list_file énumère déjà chaque chemin (dossiers ET
  # fichiers, récursivement, via find). Sans ce flag, tar re-descend dans
  # chaque dossier listé et réinsère ses enfants une deuxième fois -- déjà
  # présents avec le même contenu, ils deviennent des liens durs (dpkg refuse
  # ensuite de les installer : "error creating hard link", constaté à
  # l'installation réelle du .deb, jamais détecté par la seule comparaison de
  # SHA256 entre deux builds).
  tar --null \
    --sort=name \
    --no-recursion \
    --mtime="@${SOURCE_DATE_EPOCH}" \
    --owner=0 \
    --group=0 \
    --numeric-owner \
    -C "$root" \
    --files-from="$list_file" \
    -cf - \
  | xz -T1 -6 -c > "$output_file"
}

main() {
  require_cmd ar
  require_cmd dpkg-deb
  require_cmd find
  require_cmd sed
  require_cmd sort
  require_cmd tar
  require_cmd touch
  require_cmd xz

  local source_deb
  source_deb="$(resolve_source_deb)"

  if [[ -z "$source_deb" || ! -f "$source_deb" ]]; then
    echo "Source .deb not found" >&2
    exit 1
  fi

  if [[ -z "$OUTPUT_DEB" ]]; then
    local base_name
    base_name="$(basename "$source_deb" .deb)"
    OUTPUT_DEB="dist/repro-deb/${base_name}.deb"
  fi

  if [[ "$OUTPUT_DEB" != /* ]]; then
    OUTPUT_DEB="$(pwd)/$OUTPUT_DEB"
  fi

  local output_dir
  output_dir="$(dirname "$OUTPUT_DEB")"
  mkdir -p "$output_dir"

  local workdir
  workdir="$(mktemp -d)"
  trap 'rm -rf "${workdir:-}"' EXIT

  local extracted_root="${workdir}/root"
  mkdir -p "$extracted_root"

  log "Extracting ${source_deb}"
  dpkg-deb -R "$source_deb" "$extracted_root"
  normalize_package_tree "$extracted_root"

  local control_list="${workdir}/control.list"
  local data_list="${workdir}/data.list"

  (
    cd "${extracted_root}/DEBIAN"
    find . -mindepth 1 -print0 | sort -z | sed -z 's#^\./##'
  ) > "$control_list"

  (
    cd "$extracted_root"
    find . -mindepth 1 \( -path './DEBIAN' -o -path './DEBIAN/*' \) -prune -o -print0 | sort -z | sed -z 's#^\./##'
  ) > "$data_list"

  local control_tar="${workdir}/control.tar.xz"
  local data_tar="${workdir}/data.tar.xz"
  local debian_binary="${workdir}/debian-binary"

  log "Building normalized control archive"
  make_tar_xz "${extracted_root}/DEBIAN" "$control_list" "$control_tar"

  log "Building normalized data archive"
  make_tar_xz "$extracted_root" "$data_list" "$data_tar"

  printf '2.0\n' > "$debian_binary"

  # ar r insère/remplace des membres dans une archive EXISTANTE : si
  # OUTPUT_DEB == SOURCE_DEB (réemballage en place), écrire directement dessus
  # laisserait les anciens control.tar.gz/data.tar.gz du .deb d'origine dans
  # l'archive, à côté des nouveaux membres .xz (noms différents, donc jamais
  # remplacés). On assemble dans un fichier neuf du workdir puis on le déplace
  # en place : `mv` remplace tout le fichier, jamais un append dans l'ancien.
  local assembled_deb="${workdir}/assembled.deb"
  log "Assembling ${OUTPUT_DEB}"
  (
    cd "$workdir"
    ar rcsD "$assembled_deb" debian-binary control.tar.xz data.tar.xz
  )
  mv "$assembled_deb" "$OUTPUT_DEB"

  printf '%s\n' "$OUTPUT_DEB"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

main
