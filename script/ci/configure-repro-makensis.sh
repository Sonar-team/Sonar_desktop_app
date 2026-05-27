#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WRAPPER_DIR="${RUNNER_TEMP:-/tmp}/sonar-repro-makensis"

mkdir -p "$WRAPPER_DIR"
ln -sf "$ROOT_DIR/script/ci/makensis-repro-wrapper.sh" "$WRAPPER_DIR/makensis"

if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "$WRAPPER_DIR" >> "$GITHUB_PATH"
else
  echo "Add $WRAPPER_DIR to PATH before running Tauri NSIS builds."
fi

if [[ -n "${GITHUB_ENV:-}" ]]; then
  printf 'SONAR_REPO_ROOT=%s\n' "$ROOT_DIR" >> "$GITHUB_ENV"
fi
