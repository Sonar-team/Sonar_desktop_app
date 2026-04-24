#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
APP_NAME="${APP_NAME:-sonar}"
TAURI_BUILD_CMD="${TAURI_BUILD_CMD:-deno task tauri build}"
BIN_PATH="${BIN_PATH:-src-tauri/target/release/$APP_NAME}"
DEB_PATH="${DEB_PATH:-src-tauri/target/release/bundle/deb/${APP_NAME}_3.12.2_amd64.deb}"
WORKDIR="${WORKDIR:-/tmp/repro-check-${APP_NAME}}"
FIXED_EPOCH="${FIXED_EPOCH:-1700000000}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
  printf "%b[%s]%b %s\n" "$GREEN" "$(date +%H:%M:%S)" "$NC" "$*"
}

warn() {
  printf "%b[WARN]%b %s\n" "$YELLOW" "$NC" "$*"
}

err() {
  printf "%b[ERR]%b %s\n" "$RED" "$NC" "$*" >&2
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    err "Commande manquante : $1"
    exit 1
  }
}

clean_outputs() {
  rm -rf dist
  rm -rf src-tauri/target/release/bundle
}

run_build() {
  local mode="$1"
  local run="$2"

  log "Build mode=${mode} run=${run}"

  clean_outputs

  if [[ "$mode" == "with-flags" ]]; then
    export SOURCE_DATE_EPOCH="$FIXED_EPOCH"
    export RUSTFLAGS="--remap-path-prefix=${PROJECT_ROOT}=/workspace"
  else
    unset SOURCE_DATE_EPOCH || true
    unset RUSTFLAGS || true
  fi

  bash -lc "$TAURI_BUILD_CMD"

  if [[ ! -f "$BIN_PATH" ]]; then
    err "Binaire introuvable : $BIN_PATH"
    exit 1
  fi

  if [[ ! -f "$DEB_PATH" ]]; then
    warn "Paquet .deb introuvable : $DEB_PATH"
  fi

  local outdir="${WORKDIR}/${mode}/run${run}"
  mkdir -p "$outdir"

  cp "$BIN_PATH" "${outdir}/${APP_NAME}"
  sha256sum "${outdir}/${APP_NAME}" | tee "${outdir}/sha256-bin.txt"

  if [[ -f "$DEB_PATH" ]]; then
    cp "$DEB_PATH" "${outdir}/$(basename "$DEB_PATH")"
    sha256sum "${outdir}/$(basename "$DEB_PATH")" | tee "${outdir}/sha256-deb.txt"
  fi
}

compare_pair() {
  local mode="$1"
  local file1="$2"
  local file2="$3"
  local label="$4"

  log "Comparaison ${label} pour mode=${mode}"

  if cmp -s "$file1" "$file2"; then
    echo "RESULT ${mode} ${label}: IDENTICAL"
  else
    echo "RESULT ${mode} ${label}: DIFFERENT"
    echo "Premières différences binaires :"
    cmp -l "$file1" "$file2" | head -20 || true
  fi
}

extract_hash() {
  awk '{print $1}' "$1"
}

compare_hashes() {
  local mode="$1"
  local hash1="$2"
  local hash2="$3"
  local label="$4"

  echo "HASH ${mode} ${label} run1=${hash1}"
  echo "HASH ${mode} ${label} run2=${hash2}"

  if [[ "$hash1" == "$hash2" ]]; then
    echo "HASH_RESULT ${mode} ${label}: IDENTICAL"
  else
    echo "HASH_RESULT ${mode} ${label}: DIFFERENT"
  fi
}

run_diffoscope_if_available() {
  local file1="$1"
  local file2="$2"
  local report="$3"

  if command -v diffoscope >/dev/null 2>&1; then
    log "Lancement de diffoscope"
    diffoscope "$file1" "$file2" > "$report" || true
    echo "Rapport diffoscope généré : $report"
  else
    warn "diffoscope non installé, comparaison détaillée ignorée"
  fi
}

main() {
  require_cmd bash
  require_cmd sha256sum
  require_cmd cmp

  mkdir -p "$WORKDIR"
  cd "$PROJECT_ROOT"

  log "Projet: $PROJECT_ROOT"
  log "Binaire attendu: $BIN_PATH"
  log "Deb attendu: $DEB_PATH"
  log "Workspace: $WORKDIR"

  run_build "with-flags" "1"
  run_build "with-flags" "2"
  run_build "without-flags" "1"
  run_build "without-flags" "2"

  local bin_with_1="${WORKDIR}/with-flags/run1/${APP_NAME}"
  local bin_with_2="${WORKDIR}/with-flags/run2/${APP_NAME}"
  local bin_without_1="${WORKDIR}/without-flags/run1/${APP_NAME}"
  local bin_without_2="${WORKDIR}/without-flags/run2/${APP_NAME}"

  local deb_name
  deb_name="$(basename "$DEB_PATH")"

  local deb_with_1="${WORKDIR}/with-flags/run1/${deb_name}"
  local deb_with_2="${WORKDIR}/with-flags/run2/${deb_name}"
  local deb_without_1="${WORKDIR}/without-flags/run1/${deb_name}"
  local deb_without_2="${WORKDIR}/without-flags/run2/${deb_name}"

  compare_hashes "with-flags" \
    "$(extract_hash "${WORKDIR}/with-flags/run1/sha256-bin.txt")" \
    "$(extract_hash "${WORKDIR}/with-flags/run2/sha256-bin.txt")" \
    "binary"

  compare_hashes "without-flags" \
    "$(extract_hash "${WORKDIR}/without-flags/run1/sha256-bin.txt")" \
    "$(extract_hash "${WORKDIR}/without-flags/run2/sha256-bin.txt")" \
    "binary"

  compare_pair "with-flags" "$bin_with_1" "$bin_with_2" "binary"
  compare_pair "without-flags" "$bin_without_1" "$bin_without_2" "binary"

  echo
  echo "Comparaison du binaire avec-flags vs sans-flags :"
  sha256sum "$bin_with_1" "$bin_without_1"
  if cmp -s "$bin_with_1" "$bin_without_1"; then
    echo "RESULT cross-mode binary: IDENTICAL"
  else
    echo "RESULT cross-mode binary: DIFFERENT"
    cmp -l "$bin_with_1" "$bin_without_1" | head -20 || true
  fi

  echo
  if [[ -f "$deb_with_1" && -f "$deb_with_2" ]]; then
    compare_hashes "with-flags" \
      "$(extract_hash "${WORKDIR}/with-flags/run1/sha256-deb.txt")" \
      "$(extract_hash "${WORKDIR}/with-flags/run2/sha256-deb.txt")" \
      "deb"
    compare_pair "with-flags" "$deb_with_1" "$deb_with_2" "deb"
    run_diffoscope_if_available "$deb_with_1" "$deb_with_2" "${WORKDIR}/with-flags/diffoscope-deb.txt"
  fi

  echo
  if [[ -f "$deb_without_1" && -f "$deb_without_2" ]]; then
    compare_hashes "without-flags" \
      "$(extract_hash "${WORKDIR}/without-flags/run1/sha256-deb.txt")" \
      "$(extract_hash "${WORKDIR}/without-flags/run2/sha256-deb.txt")" \
      "deb"
    compare_pair "without-flags" "$deb_without_1" "$deb_without_2" "deb"
    run_diffoscope_if_available "$deb_without_1" "$deb_without_2" "${WORKDIR}/without-flags/diffoscope-deb.txt"
  fi

  echo
  echo "Résumé des artefacts sauvegardés dans : $WORKDIR"
  echo "  - with-flags/run1"
  echo "  - with-flags/run2"
  echo "  - without-flags/run1"
  echo "  - without-flags/run2"
}

main "$@"