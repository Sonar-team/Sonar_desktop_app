#!/usr/bin/env bash
set -euo pipefail

output="${1:-source-manifest.sha256}"
tmp_output="${output}.tmp"

git ls-files -z -- \
  ":(exclude).codex" \
  ":(exclude).codex/**" \
  ":(exclude).agents/**" \
  ":(exclude)$output" \
  ":(exclude)sbom/**" |
  xargs -0 sha256sum |
  sort -k2 > "$tmp_output"

mv "$tmp_output" "$output"
