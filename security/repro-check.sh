#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
APP_NAME="${APP_NAME:-sonar}"
TAURI_BUILD_CMD="${TAURI_BUILD_CMD:-deno task tauri build --ci --no-sign}"
BIN_PATH="${BIN_PATH:-src-tauri/target/release/$APP_NAME}"
DEB_PATH="${DEB_PATH:-}"
WORKDIR="${WORKDIR:-/tmp/repro-check-${APP_NAME}}"
FIXED_EPOCH="${FIXED_EPOCH:-1700000000}"
REQUIRE_DEB="${REQUIRE_DEB:-0}"
ALLOW_BUNDLE_NON_REPRODUCIBLE="${ALLOW_BUNDLE_NON_REPRODUCIBLE:-0}"

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

resolve_deb_path() {
  if [[ -n "$DEB_PATH" ]]; then
    printf '%s\n' "$DEB_PATH"
    return 0
  fi

  find src-tauri/target/release/bundle/deb -maxdepth 1 -type f -name '*.deb' 2>/dev/null | sort | head -n 1
}

run_build() {
  local mode="$1"
  local run="$2"

  log "Build mode=${mode} run=${run}"

  clean_outputs

  if [[ "$mode" == "with-flags" ]]; then
    SOURCE_DATE_EPOCH="$FIXED_EPOCH" \
      deno run -A ./security/repro-env.ts run bash -lc "$TAURI_BUILD_CMD"
  else
    bash -lc "$TAURI_BUILD_CMD"
  fi

  if [[ ! -f "$BIN_PATH" ]]; then
    err "Binaire introuvable : $BIN_PATH"
    exit 1
  fi

  local resolved_deb_path
  resolved_deb_path="$(resolve_deb_path)"

  if [[ -z "$resolved_deb_path" || ! -f "$resolved_deb_path" ]]; then
    if [[ "$REQUIRE_DEB" == "1" ]]; then
      err "Paquet .deb introuvable"
      exit 1
    fi
    warn "Paquet .deb introuvable"
  fi

  local outdir="${WORKDIR}/${mode}/run${run}"
  mkdir -p "$outdir"

  cp "$BIN_PATH" "${outdir}/${APP_NAME}"
  sha256sum "${outdir}/${APP_NAME}" | tee "${outdir}/sha256-bin.txt"

  if [[ -n "$resolved_deb_path" && -f "$resolved_deb_path" ]]; then
    local normalized_deb_path="${outdir}/$(basename "$resolved_deb_path")"
    ./script/package-deb-repro.sh "$resolved_deb_path" "$normalized_deb_path" >/dev/null
    sha256sum "$normalized_deb_path" | tee "${outdir}/sha256-deb.txt"
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
  log "Deb attendu: ${DEB_PATH:-auto-detect}"
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
  deb_name="$(find "${WORKDIR}/with-flags/run1" -maxdepth 1 -type f -name '*.deb' 2>/dev/null | sort | head -n 1)"
  deb_name="$(basename "$deb_name")"

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

  local repro_failed=0
  if ! cmp -s "$bin_with_1" "$bin_with_2"; then
    err "Le binaire avec flags reproductibles diffère entre les deux builds"
    repro_failed=1
  fi

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
    if ! cmp -s "$deb_with_1" "$deb_with_2"; then
      if [[ "$ALLOW_BUNDLE_NON_REPRODUCIBLE" == "1" ]]; then
        warn "Le paquet .deb avec flags reproductibles diffère entre les deux builds"
      else
        err "Le paquet .deb avec flags reproductibles diffère entre les deux builds"
        repro_failed=1
      fi
    fi
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

  if [[ "$repro_failed" != "0" ]]; then
    exit 1
  fi
}

main "$@"
