#!/usr/bin/env bash
set -euo pipefail

repo="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
release_tag="${RELEASE_TAG:-}"

empty_matrix='{"include":[]}'

emit_outputs() {
  local tag_name="$1"
  local found="$2"
  local matrix="$3"

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
      printf 'tag_name=%s\n' "$tag_name"
      printf 'found=%s\n' "$found"
      printf 'matrix=%s\n' "$matrix"
    } >> "$GITHUB_OUTPUT"
  else
    printf '%s\n' "$matrix"
  fi
}

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

if [[ -n "$release_tag" ]]; then
  release_api_path="repos/${repo}/releases/tags/${release_tag}"
else
  release_api_path="repos/${repo}/releases/latest"
fi

if ! gh api "$release_api_path" > "${tmpdir}/release.json"; then
  echo "No release found for ${release_tag:-latest release}" >&2
  emit_outputs "${release_tag}" "false" "$empty_matrix"
  exit 0
fi

tag_name="$(jq -r '.tag_name // empty' "${tmpdir}/release.json")"
if [[ -z "$tag_name" ]]; then
  echo "Release response does not contain tag_name" >&2
  exit 1
fi

matrix="$(
  jq -c --arg tag "$tag_name" '
    def scannable_name:
      test("\\.(deb|rpm|msi|dmg)$"; "i") or test("(^|[-_.])setup\\.exe$"; "i");

    {
      include: [
        (.assets // [])[]
        | select((.state // "") == "uploaded")
        | select(.name | scannable_name)
        | {
            tag_name: $tag,
            asset_id: (.id | tostring),
            asset_name: .name,
            safe_name: (.name | gsub("[^A-Za-z0-9._-]"; "-"))
          }
      ]
    }
  ' "${tmpdir}/release.json"
)"

asset_count="$(jq '.include | length' <<< "$matrix")"
if [[ "$asset_count" -eq 0 ]]; then
  echo "No scannable release artifacts found for ${tag_name}" >&2
  emit_outputs "$tag_name" "false" "$matrix"
  exit 0
fi

echo "Found ${asset_count} scannable release artifact(s) for ${tag_name}" >&2
emit_outputs "$tag_name" "true" "$matrix"
