#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

if ! command -v shellcheck >/dev/null 2>&1; then
  echo "shellcheck is required (CI uses version 0.11.0)" >&2
  exit 1
fi

scripts=(release_downloads.sh)
while IFS= read -r -d '' script_path; do
  scripts+=("$script_path")
done < <(find script security -type f -name '*.sh' -print0 | sort -z)

shellcheck --external-sources --severity=warning "${scripts[@]}"
