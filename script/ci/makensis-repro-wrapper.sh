#!/usr/bin/env bash
set -euo pipefail

repo_root="${SONAR_REPO_ROOT:-$PWD}"
wrapper_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"

find_real_makensis() {
  if [[ -n "${SONAR_REAL_MAKENSIS:-}" ]]; then
    printf '%s\n' "$SONAR_REAL_MAKENSIS"
    return 0
  fi

  IFS=':' read -r -a path_entries <<< "$PATH"
  for entry in "${path_entries[@]}"; do
    local entry_dir
    entry_dir="$(cd "$entry" 2>/dev/null && pwd -P || true)"
    if [[ -z "$entry" || "$entry_dir" == "$wrapper_dir" ]]; then
      continue
    fi

    for candidate in "$entry/makensis" "$entry/makensis.exe"; do
      if [[ -x "$candidate" ]]; then
        printf '%s\n' "$candidate"
        return 0
      fi
    done
  done

  return 1
}

if ! real_makensis="$(find_real_makensis)"; then
  echo "Unable to locate the real makensis executable behind the reproducible wrapper." >&2
  exit 127
fi

if [[ -n "${SOURCE_DATE_EPOCH:-}" ]]; then
  deno run -A "${repo_root}/script/ci/normalize-bundle-inputs.ts"
fi

exec "$real_makensis" "$@"
