#!/usr/bin/env bash
set -euo pipefail

platform="${1:?usage: upload-sigstore-bundles.sh <platform> [target-dir]}"
release_tag="${GITHUB_REF_NAME:?GITHUB_REF_NAME is required}"
signature_dir="release-signatures"

mapfile -t sigstore_bundles < <(
  find "$signature_dir" -type f -name "${platform}-*.sigstore.json" | sort
)

test "${#sigstore_bundles[@]}" -gt 0

gh release upload "$release_tag" \
  "${sigstore_bundles[@]}" \
  --clobber
