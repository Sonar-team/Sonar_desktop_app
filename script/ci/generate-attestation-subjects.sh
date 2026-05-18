#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: generate-attestation-subjects.sh <platform> [target-dir] [output-file]}"
target_dir="${2:-src-tauri/target}"
output_file="${3:-release-attestation-subjects-${platform}.sha256}"
hashes_file="release-hashes-${platform}.md"

if command -v sha256sum >/dev/null 2>&1; then
  hash_cmd=(sha256sum)
else
  hash_cmd=(shasum -a 256)
fi

write_subject_checksum() {
  local artifact="${1:?artifact path is required}"
  local digest

  digest="$("${hash_cmd[@]}" "$artifact" | awk '{print $1}')"

  if ! [[ "$digest" =~ ^[0-9a-fA-F]{64}$ ]]; then
    echo "invalid SHA256 digest for ${artifact}" >&2
    return 1
  fi

  printf '%s  %s\n' "$digest" "$artifact"
}

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

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

{
  while IFS= read -r artifact; do
    write_subject_checksum "$artifact"
  done < "${tmpdir}/binary-artifacts.txt"

  while IFS= read -r artifact; do
    write_subject_checksum "$artifact"
  done < "${tmpdir}/bundle-artifacts.txt"

  write_subject_checksum "$hashes_file"
} > "$output_file"

grep -Eq '^[0-9a-f]{64}  ' "$output_file"
