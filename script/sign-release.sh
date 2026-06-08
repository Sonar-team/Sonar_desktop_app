#!/usr/bin/env bash
set -euo pipefail

artifact="$1"

cosign sign-blob \
  --yes \
  --bundle "${artifact}.sigstore.json" \
  "$artifact"