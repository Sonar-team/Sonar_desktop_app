#!/usr/bin/env bash
set -euo pipefail

SOURCE_APP="${1:-}"
OUTPUT_DMG="${2:-}"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-1700000000}"
VOLUME_NAME="${VOLUME_NAME:-SONAR}"

usage() {
  cat <<'EOF'
usage: script/package-dmg-repro.sh [source.app] [output.dmg]

Build a normalized macOS DMG from an unsigned .app bundle.
The script must run on macOS because it relies on hdiutil.
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

resolve_source_app() {
  if [[ -n "$SOURCE_APP" ]]; then
    printf '%s\n' "$SOURCE_APP"
    return 0
  fi

  find src-tauri/target -type d -name '*.app' 2>/dev/null | sort | head -n 1
}

timestamp_arg() {
  date -u -r "$SOURCE_DATE_EPOCH" '+%Y%m%d%H%M.%S'
}

normalize_tree() {
  local root="$1"
  local timestamp="$2"

  if command -v xattr >/dev/null 2>&1; then
    xattr -cr "$root" || true
  fi

  find "$root" -type d -print | LC_ALL=C sort | while IFS= read -r dir; do
    chmod 755 "$dir"
    touch -h -t "$timestamp" "$dir"
  done

  find "$root" -type f -print | LC_ALL=C sort | while IFS= read -r file; do
    if [[ -x "$file" ]]; then
      chmod 755 "$file"
    else
      chmod 644 "$file"
    fi
    touch -h -t "$timestamp" "$file"
  done

  find "$root" -type l -print | LC_ALL=C sort | while IFS= read -r link; do
    touch -h -t "$timestamp" "$link"
  done
}

main() {
  require_cmd date
  require_cmd ditto
  require_cmd find
  require_cmd hdiutil
  require_cmd sort
  require_cmd touch

  local source_app
  source_app="$(resolve_source_app)"

  if [[ -z "$source_app" || ! -d "$source_app" ]]; then
    echo "Source .app not found" >&2
    exit 1
  fi

  if [[ -z "$OUTPUT_DMG" ]]; then
    local base_name
    base_name="$(basename "$source_app" .app)"
    OUTPUT_DMG="dist/repro-dmg/${base_name}.dmg"
  fi

  if [[ "$OUTPUT_DMG" != /* ]]; then
    OUTPUT_DMG="$(pwd)/$OUTPUT_DMG"
  fi

  local output_dir
  output_dir="$(dirname "$OUTPUT_DMG")"
  mkdir -p "$output_dir"

  local workdir
  workdir="$(mktemp -d)"
  trap 'rm -rf "${workdir:-}"' EXIT

  local staging="${workdir}/staging"
  local rw_dmg="${workdir}/staging.dmg"
  local timestamp
  timestamp="$(timestamp_arg)"

  mkdir -p "$staging"

  log "Copying $(basename "$source_app")"
  ditto --noextattr --noqtn "$source_app" "${staging}/$(basename "$source_app")"
  ln -s /Applications "${staging}/Applications"

  log "Normalizing app bundle metadata"
  normalize_tree "$staging" "$timestamp"

  rm -f "$OUTPUT_DMG"

  log "Creating writable DMG"
  hdiutil create \
    -quiet \
    -ov \
    -fs HFS+ \
    -format UDRW \
    -volname "$VOLUME_NAME" \
    -srcfolder "$staging" \
    "$rw_dmg"

  log "Converting compressed DMG"
  hdiutil convert \
    "$rw_dmg" \
    -quiet \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "$OUTPUT_DMG"

  touch -h -t "$timestamp" "$OUTPUT_DMG"
  printf '%s\n' "$OUTPUT_DMG"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

main
