#!/usr/bin/env bash
set -euo pipefail

binary_path="${1:?usage: smoke-test-release-binary.sh <binary-path> [timeout-seconds]}"
timeout_seconds="${2:-30}"

if [[ ! -f "$binary_path" ]]; then
  echo "Release binary not found: $binary_path" >&2
  exit 1
fi

case "$binary_path" in
  *.exe) ;;
  *) chmod +x "$binary_path" ;;
esac

tmp_dir="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
stdout_log="$(mktemp "${tmp_dir%/}/sonar-smoke-stdout.XXXXXX.log")"
file_log="$(mktemp "${tmp_dir%/}/sonar-smoke-file.XXXXXX.log")"
combined_log="$(mktemp "${tmp_dir%/}/sonar-smoke-combined.XXXXXX.log")"
trap 'rm -f "$stdout_log" "$file_log" "$combined_log"' EXIT

echo "Smoke testing release binary: $binary_path"

SONAR_SMOKE_LOG_PATH="$file_log" "$binary_path" --sonar-smoke-test >"$stdout_log" 2>&1 &
pid="$!"
deadline=$((SECONDS + timeout_seconds))

while kill -0 "$pid" 2>/dev/null; do
  if (( SECONDS >= deadline )); then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    echo "Smoke test timed out after ${timeout_seconds}s" >&2
    exit 1
  fi
  sleep 1
done

set +e
wait "$pid"
status="$?"
set -e

if [[ -s "$stdout_log" ]]; then
  cat "$stdout_log"
fi

if [[ -s "$file_log" ]] && ! cmp -s "$stdout_log" "$file_log"; then
  cat "$file_log"
fi

cat "$stdout_log" "$file_log" > "$combined_log"

if [[ "$status" -ne 0 ]]; then
  echo "Smoke test command failed with exit code $status" >&2
  exit "$status"
fi

required_logs=(
  "Using device "
  "SONAR_STARTUP_VALIDATION=OK"
)

for required_log in "${required_logs[@]}"; do
  if ! grep -Fq "$required_log" "$combined_log"; then
    echo "Missing required smoke log: $required_log" >&2
    exit 1
  fi
done

echo "Release binary smoke logs validated."
