#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

release_tag="${1:?usage: download-signed-release-for-e2e.sh <release-tag> <output-dir> [platform]}"
output_dir="${2:?usage: download-signed-release-for-e2e.sh <release-tag> <output-dir> [platform]}"
platform="${3:-ubuntu-22.04}"
repository="${SONAR_RELEASE_REPOSITORY:-Sonar-team/Sonar_desktop_app}"
wait_seconds="${SONAR_RELEASE_WAIT_SECONDS:-0}"
poll_seconds="${SONAR_RELEASE_POLL_SECONDS:-15}"
trusted_root="${SONAR_SIGSTORE_TRUSTED_ROOT:-${ROOT_DIR}/security/sigstore-trusted-root.json}"
expected_commit="${SONAR_EXPECTED_COMMIT:-}"

if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid release tag: $release_tag" >&2
  exit 1
fi
if [[ ! "$wait_seconds" =~ ^[0-9]+$ || ! "$poll_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "Release wait values must be positive integers" >&2
  exit 1
fi
if [[ -n "$expected_commit" && ! "$expected_commit" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Invalid expected Git commit: $expected_commit" >&2
  exit 1
fi

for command_name in gh cosign tar find jq; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command not found: $command_name" >&2
    exit 1
  fi
done
[[ -f "$trusted_root" ]] || {
  echo "Trusted root not found: $trusted_root" >&2
  exit 1
}

if [[ -e "$output_dir" ]] &&
  find "$output_dir" -mindepth 1 -print -quit 2>/dev/null | grep -q .; then
  echo "Output directory is not empty: $output_dir" >&2
  exit 1
fi
mkdir -p "$output_dir/download" "$output_dir/extracted"
output_dir="$(cd "$output_dir" && pwd)"

deadline=$((SECONDS + wait_seconds))
while true; do
  release_json="$(
    gh release view "$release_tag" --repo "$repository" \
      --json tagName,isDraft 2>/dev/null || true
  )"
  if [[ -n "$release_json" ]] &&
    [[ "$(jq -r '.tagName' <<< "$release_json")" == "$release_tag" ]] &&
    [[ "$(jq -r '.isDraft' <<< "$release_json")" == "false" ]]; then
    break
  fi

  if ((SECONDS >= deadline)); then
    echo "Signed public release not available: ${repository}@${release_tag}" >&2
    exit 1
  fi
  printf 'Waiting for signed release %s (%ss remaining)\n' \
    "$release_tag" "$((deadline - SECONDS))" >&2
  sleep "$poll_seconds"
done

version="${release_tag#v}"
archive_name="sonar-offline-kit-${version}-${platform}.tar.gz"
bundle_name="${platform}-${archive_name}.sigstore.json"

gh release download "$release_tag" \
  --repo "$repository" \
  --dir "$output_dir/download" \
  --pattern "$archive_name" \
  --pattern "$bundle_name"

archive_path="$output_dir/download/$archive_name"
bundle_path="$output_dir/download/$bundle_name"
[[ -f "$archive_path" && -f "$bundle_path" ]] || {
  echo "Offline kit or signature bundle is missing from the release" >&2
  exit 1
}

verification_args=(
  --archive "$archive_path"
  --bundle "$bundle_path"
  --trusted-root "$trusted_root"
  --expected-tag "$release_tag"
  --platform "$platform"
  --extract-to "$output_dir/extracted"
)
if [[ -n "$expected_commit" ]]; then
  verification_args+=(--expected-commit "$expected_commit")
fi

verification_output="$({
  "$ROOT_DIR/script/ci/verify-offline-release-kit.sh" \
    "${verification_args[@]}"
} 2> >(tee "$output_dir/verification.log" >&2))"
printf '%s\n' "$verification_output" | tee -a "$output_dir/verification.log" >&2

signed_deb="$(
  awk -F= '$1 == "SONAR_SIGNED_DEB" { sub(/^[^=]*=/, ""); print }' \
    <<< "$verification_output"
)"
[[ -n "$signed_deb" && -f "$signed_deb" ]] || {
  echo "The verified Linux kit contains no unique Debian package" >&2
  exit 1
}

printf '%s\n' "$signed_deb"
