#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: sign-release-artifacts.sh <platform> [artifact-dir]}"
target_dir="${2:-release-artifacts}"
hashes_file="release-hashes-${platform}.md"
signature_dir="release-signatures"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
mkdir -p "$signature_dir"

find "$target_dir" -type f \
  \( -name 'sonar' -o -name 'sonar.exe' -o -name 'sonar-*' -o -name 'sonar-*.exe' \) \
  ! -path '*/bundle/*' \
  | sort > "${tmpdir}/binary-artifacts.txt"

test -s "${tmpdir}/binary-artifacts.txt"
test -f "$hashes_file"

sign_artifact() {
  local artifact="${1:?artifact path is required}"
  local artifact_name

  artifact_name="$(basename "$artifact")"

  cosign sign-blob --yes \
    --bundle "${signature_dir}/${platform}-${artifact_name}.sigstore.json" \
    "$artifact"
}

while IFS= read -r artifact; do
  sign_artifact "$artifact"
done < "${tmpdir}/binary-artifacts.txt"

sign_artifact "$hashes_file"
