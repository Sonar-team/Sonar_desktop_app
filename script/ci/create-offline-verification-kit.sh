#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

platform="${1:?usage: create-offline-verification-kit.sh <platform> [artifact-dir] [signature-dir] [output-dir]}"
artifact_dir="${2:-release-artifacts}"
signature_dir="${3:-release-signatures}"
output_dir="${4:-offline-kits}"
release_tag="${GITHUB_REF_NAME:?GITHUB_REF_NAME is required}"
repository="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
source_commit="${GITHUB_SHA:?GITHUB_SHA is required}"
cosign_version="${COSIGN_VERSION:?COSIGN_VERSION is required}"
trusted_root_sha256="${SIGSTORE_TRUSTED_ROOT_SHA256:?SIGSTORE_TRUSTED_ROOT_SHA256 is required}"
oidc_issuer="https://token.actions.githubusercontent.com"
workflow_identity="https://github.com/${repository}/.github/workflows/publish.yml@refs/tags/${release_tag}"
trusted_root="${ROOT_DIR}/security/sigstore-trusted-root.json"
release_hashes="release-hashes-${platform}.md"

if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Unsupported release tag: $release_tag" >&2
  exit 1
fi

if [[ ! "$repository" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
  echo "Invalid GitHub repository identifier: $repository" >&2
  exit 1
fi
if [[ ! "$source_commit" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Invalid GitHub source commit: $source_commit" >&2
  exit 1
fi

for command_name in cosign tar find cp awk grep tr; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command not found: $command_name" >&2
    exit 1
  fi
done

if command -v sha256sum >/dev/null 2>&1; then
  hash_cmd=(sha256sum)
else
  hash_cmd=(shasum -a 256)
fi

for required_path in "$artifact_dir" "$signature_dir" "$trusted_root" "$release_hashes"; do
  if [[ ! -e "$required_path" ]]; then
    echo "Required release input not found: $required_path" >&2
    exit 1
  fi
done

actual_trusted_root_sha256="$("${hash_cmd[@]}" "$trusted_root" | awk '{ print $1 }')"
if [[ "$actual_trusted_root_sha256" != "$trusted_root_sha256" ]]; then
  echo "The versioned Sigstore trusted root checksum is invalid" >&2
  exit 1
fi

cosign_version_output="$(cosign version 2>&1 | tr -d '\r')"
if ! grep -Eq "GitVersion:[[:space:]]+v?${cosign_version}$" <<< "$cosign_version_output"; then
  echo "Installed Cosign does not match COSIGN_VERSION=$cosign_version" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

version="${release_tag#v}"
kit_name="sonar-offline-kit-${version}-${platform}"
kit_root="${tmpdir}/${kit_name}"
archive_name="${kit_name}.tar.gz"
archive_path="${output_dir}/${archive_name}"
archive_bundle="${signature_dir}/${platform}-${archive_name}.sigstore.json"

mkdir -p \
  "$kit_root/artifacts" \
  "$kit_root/hashes" \
  "$kit_root/signatures" \
  "$kit_root/tools" \
  "$kit_root/trust" \
  "$kit_root/verify" \
  "$output_dir"

artifact_count=0
while IFS= read -r artifact; do
  artifact_name="$(basename "$artifact")"
  artifact_bundle="${signature_dir}/${platform}-${artifact_name}.sigstore.json"
  if [[ ! -f "$artifact_bundle" ]]; then
    echo "Sigstore bundle not found for $artifact_name: $artifact_bundle" >&2
    exit 1
  fi

  cp "$artifact" "$kit_root/artifacts/$artifact_name"
  cp "$artifact_bundle" "$kit_root/signatures/"
  artifact_count=$((artifact_count + 1))
done < <(find "$artifact_dir" -maxdepth 1 -type f | LC_ALL=C sort)

if ((artifact_count == 0)); then
  echo "No release artifact found in $artifact_dir" >&2
  exit 1
fi

hashes_bundle="${signature_dir}/${platform}-$(basename "$release_hashes").sigstore.json"
if [[ ! -f "$hashes_bundle" ]]; then
  echo "Sigstore bundle not found for $release_hashes: $hashes_bundle" >&2
  exit 1
fi
cp "$release_hashes" "$kit_root/hashes/"
cp "$hashes_bundle" "$kit_root/signatures/"
cp "$trusted_root" "$kit_root/trust/sigstore-trusted-root.json"
cp "$ROOT_DIR/script/ci/verify-offline-release-kit.sh" "$kit_root/verify/"
cp "$ROOT_DIR/script/ci/verify-offline-release-kit.ps1" "$kit_root/verify/"

cosign_source="$(command -v cosign)"
case "$platform" in
  windows-*) cosign_name="cosign.exe" ;;
  *) cosign_name="cosign" ;;
esac
cp "$cosign_source" "$kit_root/tools/$cosign_name"
chmod 0755 "$kit_root/tools/$cosign_name" "$kit_root/verify/verify-offline-release-kit.sh"
cosign version > "$kit_root/tools/cosign-version.txt"

if [[ "$platform" == "ubuntu-22.04" ]]; then
  for validation_path in \
    "$ROOT_DIR/dist/assets" \
    "$ROOT_DIR/script/e2e/run-sonar-x11-e2e.sh" \
    "$ROOT_DIR/script/e2e/sonar-x11-input.c" \
    "$ROOT_DIR/script/ci/smoke-test-release-binary.sh" \
    "$ROOT_DIR/src-tauri/test_files/pcaps/import/pcap_tshark_corpus/industrial_ethernet.pcap" \
    "$ROOT_DIR/src-tauri/test_files/20260703_NP_Matrice.csv" \
    "$ROOT_DIR/src-tauri/test_files/20260703_NP_Labels.csv"; do
    if [[ ! -e "$validation_path" ]]; then
      echo "Linux validation payload not found: $validation_path" >&2
      exit 1
    fi
  done

  mkdir -p \
    "$kit_root/validation/script/ci" \
    "$kit_root/validation/script/e2e" \
    "$kit_root/validation/fixtures"
  cp -R "$ROOT_DIR/dist" "$kit_root/validation/dist"
  cp "$ROOT_DIR/script/e2e/run-sonar-x11-e2e.sh" \
    "$ROOT_DIR/script/e2e/sonar-x11-input.c" \
    "$kit_root/validation/script/e2e/"
  cp "$ROOT_DIR/script/ci/smoke-test-release-binary.sh" \
    "$kit_root/validation/script/ci/"
  cp "$ROOT_DIR/src-tauri/test_files/pcaps/import/pcap_tshark_corpus/industrial_ethernet.pcap" \
    "$kit_root/validation/fixtures/industrial_ethernet.pcap"
  cp "$ROOT_DIR/src-tauri/test_files/20260703_NP_Matrice.csv" \
    "$kit_root/validation/fixtures/matrice.csv"
  cp "$ROOT_DIR/src-tauri/test_files/20260703_NP_Labels.csv" \
    "$kit_root/validation/fixtures/labels.csv"
  validation_payload="present"
else
  validation_payload="absent"
fi

{
  printf 'format_version=1\n'
  printf 'repository=%s\n' "$repository"
  printf 'release_tag=%s\n' "$release_tag"
  printf 'source_commit=%s\n' "$source_commit"
  printf 'platform=%s\n' "$platform"
  printf 'certificate_identity=%s\n' "$workflow_identity"
  printf 'certificate_oidc_issuer=%s\n' "$oidc_issuer"
  printf 'cosign_version=%s\n' "$cosign_version"
  printf 'validation_payload=%s\n' "$validation_payload"
} > "$kit_root/metadata.env"

(
  cd "$kit_root"
  while IFS= read -r kit_file; do
    "${hash_cmd[@]}" "${kit_file#./}"
  done < <(find . -type f ! -name SHA256SUMS | LC_ALL=C sort)
) > "$kit_root/SHA256SUMS"

tar -czf "$archive_path" -C "$tmpdir" "$kit_name"

cosign sign-blob --yes \
  --bundle "$archive_bundle" \
  "$archive_path"

offline_hashes="release-hashes-${platform}-offline-kit.md"
{
  printf '### %s offline verification kit\n\n' "$platform"
  printf '#### Artifacts\n\n'
  "${hash_cmd[@]}" "$archive_path"
} > "$offline_hashes"

test -s "$archive_path"
test -s "$archive_bundle"
grep -Eq '^[0-9a-f]{64} ' "$offline_hashes"

printf '%s\n' "$archive_path"
