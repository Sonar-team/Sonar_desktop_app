#!/usr/bin/env bash
set -euo pipefail

output="${1:-source-manifest.sha256}"
tmp_output="${output}.tmp"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sonar-source-manifest.XXXXXX")"
trap 'rm -rf "$tmp_dir" "$tmp_output"' EXIT

tree="$(git write-tree)"
git archive "$tree" | tar -x -C "$tmp_dir"

rel_output="${output#./}"

(
  cd "$tmp_dir"
  find . -type f \
    ! -path './.codex/*' \
    ! -path './.agents/*' \
    ! -path './sbom/*' \
    ! -path "./$rel_output" \
    -print0 |
    sort -z |
    while IFS= read -r -d '' path; do
      rel_path="${path#./}"
      sha256sum "$rel_path"
    done
) > "$tmp_output"

mv "$tmp_output" "$output"
