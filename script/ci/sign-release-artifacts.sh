#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: sign-release-artifacts.sh <platform> [target-dir]}"
target_dir="${2:-src-tauri/target}"
hashes_file="release-hashes-${platform}.md"
signature_dir="release-signatures"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
mkdir -p "$signature_dir"

find "$target_dir" -type f \
  \( -path '*/release/sonar' -o -path '*/release/sonar.exe' \) \
  ! -path '*/bundle/*' \
  | sort > "${tmpdir}/binary-artifacts.txt"

find "$target_dir" -type f \
  \( -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' -o -name '*.dmg' -o -name '*.msi' -o -name '*.exe' \) \
  -path '*/bundle/*' \
  | sort > "${tmpdir}/bundle-artifacts.txt"

test -s "${tmpdir}/binary-artifacts.txt"
test -s "${tmpdir}/bundle-artifacts.txt"
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

while IFS= read -r artifact; do
  sign_artifact "$artifact"
done < "${tmpdir}/bundle-artifacts.txt"

sign_artifact "$hashes_file"
