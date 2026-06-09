#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: generate-attestation-subjects.sh <platform> [artifact-dir] [output-file]}"
target_dir="${2:-release-artifacts}"
output_file="${3:-release-attestation-subjects-${platform}.txt}"
hashes_file="release-hashes-${platform}.md"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

find "$target_dir" -type f \
  \( -name 'sonar' -o -name 'sonar.exe' -o -name 'sonar-*' -o -name 'sonar-*.exe' \) \
  ! -path '*/bundle/*' \
  | sort > "${tmpdir}/binary-artifacts.txt"

test -s "${tmpdir}/binary-artifacts.txt"
test -f "$hashes_file"

{
  cat "${tmpdir}/binary-artifacts.txt"
  printf '%s\n' "$hashes_file"
} > "$output_file"

test -s "$output_file"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo 'subject-paths<<EOF'
    cat "$output_file"
    echo EOF
  } >> "$GITHUB_OUTPUT"
fi
