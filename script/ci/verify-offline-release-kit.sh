#!/usr/bin/env bash
set -euo pipefail

EXPECTED_REPOSITORY="Sonar-team/Sonar_desktop_app"
EXPECTED_OIDC_ISSUER="https://token.actions.githubusercontent.com"

archive_path=""
bundle_path=""
trusted_root=""
expected_tag=""
expected_commit=""
platform="ubuntu-22.04"
extract_dir=""
cosign_binary="${COSIGN_BIN:-}"

usage() {
  cat <<'EOF'
Usage: verify-offline-release-kit.sh \
  --archive KIT.tar.gz \
  --bundle KIT.tar.gz.sigstore.json \
  --trusted-root sigstore-trusted-root.json \
  --expected-tag vX.Y.Z \
  [--expected-commit GIT_SHA] \
  --extract-to DIR \
  [--platform ubuntu-22.04] \
  [--cosign PATH]

La racine de confiance et ce script doivent provenir de l'image de confiance,
pas de l'archive encore non vérifiée.
EOF
}

fail() {
  printf 'SONAR offline verification failed: %s\n' "$*" >&2
  exit 1
}

while (($# > 0)); do
  case "$1" in
    --archive)
      archive_path="${2:?--archive expects a path}"
      shift 2
      ;;
    --bundle)
      bundle_path="${2:?--bundle expects a path}"
      shift 2
      ;;
    --trusted-root)
      trusted_root="${2:?--trusted-root expects a path}"
      shift 2
      ;;
    --expected-tag)
      expected_tag="${2:?--expected-tag expects a tag}"
      shift 2
      ;;
    --platform)
      platform="${2:?--platform expects a value}"
      shift 2
      ;;
    --expected-commit)
      expected_commit="${2:?--expected-commit expects a Git SHA}"
      shift 2
      ;;
    --extract-to)
      extract_dir="${2:?--extract-to expects a path}"
      shift 2
      ;;
    --cosign)
      cosign_binary="${2:?--cosign expects a path}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown option: $1"
      ;;
  esac
done

for required_value in archive_path bundle_path trusted_root expected_tag extract_dir; do
  if [[ -z "${!required_value}" ]]; then
    fail "missing required option: ${required_value}"
  fi
done

if [[ ! "$expected_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  fail "invalid expected tag: $expected_tag"
fi
if [[ ! "$platform" =~ ^[A-Za-z0-9_.-]+$ ]]; then
  fail "invalid platform: $platform"
fi
if [[ -n "$expected_commit" && ! "$expected_commit" =~ ^[0-9a-f]{40}$ ]]; then
  fail "invalid expected Git commit: $expected_commit"
fi

for required_file in "$archive_path" "$bundle_path" "$trusted_root"; do
  [[ -f "$required_file" ]] || fail "required file not found: $required_file"
done

if [[ -z "$cosign_binary" ]]; then
  cosign_binary="$(command -v cosign || true)"
fi
[[ -n "$cosign_binary" && -x "$cosign_binary" ]] ||
  fail "a pre-trusted cosign executable is required"

for command_name in tar find grep awk cmp; do
  command -v "$command_name" >/dev/null 2>&1 ||
    fail "required command not found: $command_name"
done

if command -v sha256sum >/dev/null 2>&1; then
  hash_cmd=(sha256sum)
  hash_check_cmd=(sha256sum --check)
else
  hash_cmd=(shasum -a 256)
  hash_check_cmd=(shasum -a 256 --check)
fi

if [[ -e "$extract_dir" ]] &&
  find "$extract_dir" -mindepth 1 -print -quit 2>/dev/null | grep -q .; then
  fail "extraction directory is not empty: $extract_dir"
fi
mkdir -p "$extract_dir"
extract_dir="$(cd "$extract_dir" && pwd)"

if [[ "$extract_dir" == "/" ]]; then
  fail "refusing to extract into /"
fi

workflow_identity="https://github.com/${EXPECTED_REPOSITORY}/.github/workflows/publish.yml@refs/tags/${expected_tag}"
verification_policy=(
  --trusted-root "$trusted_root"
  --certificate-identity "$workflow_identity"
  --certificate-oidc-issuer "$EXPECTED_OIDC_ISSUER"
)
if [[ -n "$expected_commit" ]]; then
  verification_policy+=(--certificate-github-workflow-sha "$expected_commit")
fi

"$cosign_binary" verify-blob \
  "${verification_policy[@]}" \
  --bundle "$bundle_path" \
  "$archive_path" >&2 || fail "offline kit archive signature is invalid"

archive_list="$(mktemp)"
archive_verbose_list="$(mktemp)"
trap 'rm -f "$archive_list" "$archive_verbose_list"' EXIT
tar -tzf "$archive_path" > "$archive_list"
tar -tvzf "$archive_path" > "$archive_verbose_list"

if grep -Eq '(^/|(^|/)\.\.(/|$))' "$archive_list"; then
  fail "archive contains an unsafe path"
fi
if awk '$1 !~ /^[-d]/ { unsafe = 1 } END { exit unsafe ? 0 : 1 }' \
  "$archive_verbose_list"; then
  fail "archive contains a symbolic link or another unsupported entry"
fi

tar -xzf "$archive_path" -C "$extract_dir"

version="${expected_tag#v}"
kit_name="sonar-offline-kit-${version}-${platform}"
kit_root="${extract_dir}/${kit_name}"
[[ -d "$kit_root" ]] || fail "expected kit directory not found: $kit_name"

top_level_count="$(find "$extract_dir" -mindepth 1 -maxdepth 1 | wc -l | tr -d '[:space:]')"
[[ "$top_level_count" -eq 1 ]] || fail "archive must contain exactly one top-level directory"

for kit_file in \
  "$kit_root/metadata.env" \
  "$kit_root/SHA256SUMS" \
  "$kit_root/hashes/release-hashes-${platform}.md" \
  "$kit_root/signatures/${platform}-release-hashes-${platform}.md.sigstore.json" \
  "$kit_root/trust/sigstore-trusted-root.json"; do
  [[ -f "$kit_file" ]] || fail "kit file not found: $kit_file"
done

cmp -s "$trusted_root" "$kit_root/trust/sigstore-trusted-root.json" ||
  fail "the trusted root embedded in the kit differs from the provisioned root"

(
  cd "$kit_root"
  "${hash_check_cmd[@]}" SHA256SUMS
) >&2 || fail "the internal kit checksum manifest is invalid"

metadata="$kit_root/metadata.env"
grep -Fxq 'format_version=1' "$metadata" || fail "unsupported kit format"
grep -Fxq "repository=${EXPECTED_REPOSITORY}" "$metadata" || fail "unexpected repository"
grep -Fxq "release_tag=${expected_tag}" "$metadata" || fail "unexpected release tag"
grep -Fxq "platform=${platform}" "$metadata" || fail "unexpected platform"
grep -Fxq "certificate_identity=${workflow_identity}" "$metadata" || fail "unexpected signing identity"
grep -Fxq "certificate_oidc_issuer=${EXPECTED_OIDC_ISSUER}" "$metadata" || fail "unexpected OIDC issuer"

source_commit="$(awk -F= '$1 == "source_commit" { print $2 }' "$metadata")"
[[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || fail "invalid source commit metadata"
if [[ -n "$expected_commit" && "$source_commit" != "$expected_commit" ]]; then
  fail "the kit source commit differs from the expected Git commit"
fi

cosign_version="$(awk -F= '$1 == "cosign_version" { print $2 }' "$metadata")"
[[ "$cosign_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "invalid cosign version metadata"
"$cosign_binary" version 2>&1 |
  grep -Eq "GitVersion:[[:space:]]+v?${cosign_version}$" ||
  fail "the verifier Cosign version does not match the kit"

release_hashes="$kit_root/hashes/release-hashes-${platform}.md"
release_hashes_bundle="$kit_root/signatures/${platform}-release-hashes-${platform}.md.sigstore.json"
"$cosign_binary" verify-blob \
  "${verification_policy[@]}" \
  --bundle "$release_hashes_bundle" \
  "$release_hashes" >&2 || fail "release hash manifest signature is invalid"

artifact_count=0
while IFS= read -r artifact; do
  artifact_name="$(basename "$artifact")"
  artifact_bundle="$kit_root/signatures/${platform}-${artifact_name}.sigstore.json"
  [[ -f "$artifact_bundle" ]] || fail "signature missing for $artifact_name"

  "$cosign_binary" verify-blob \
    "${verification_policy[@]}" \
    --bundle "$artifact_bundle" \
    "$artifact" >&2 || fail "signature invalid for $artifact_name"

  artifact_digest="$("${hash_cmd[@]}" "$artifact" | awk '{ print $1 }')"
  awk -v digest="$artifact_digest" -v path="release-artifacts/${artifact_name}" '
    $1 == digest {
      candidate = $2
      sub(/^\*/, "", candidate)
      if (candidate == path) {
        found = 1
      }
    }
    END { exit found ? 0 : 1 }
  ' \
    "$release_hashes" || fail "signed hash missing for $artifact_name"
  artifact_count=$((artifact_count + 1))
done < <(find "$kit_root/artifacts" -maxdepth 1 -type f | LC_ALL=C sort)

((artifact_count > 0)) || fail "the kit contains no release artifact"

printf 'SONAR_OFFLINE_KIT_VERIFIED=%s\n' "$kit_root"

deb_count="$(find "$kit_root/artifacts" -maxdepth 1 -type f -name '*.deb' | wc -l | tr -d '[:space:]')"
if [[ "$deb_count" -eq 1 ]]; then
  deb_path="$(find "$kit_root/artifacts" -maxdepth 1 -type f -name '*.deb' -print)"
  printf 'SONAR_SIGNED_DEB=%s\n' "$deb_path"
fi
