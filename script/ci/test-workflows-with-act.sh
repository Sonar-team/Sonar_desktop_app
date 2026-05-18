#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ACT_BIN="${ACT_BIN:-act}"
ACT_IMAGE_UBUNTU="${ACT_IMAGE_UBUNTU:-catthehacker/ubuntu:act-22.04}"
SECRETS_FILE="${ACT_SECRETS_FILE:-.secrets.act}"

cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
usage: test-workflows-with-act.sh [--validate-only]

Runs act validation for all tracked workflows, then executes the workflows that
are practical to run locally under act. Windows and macOS jobs are validated
but not executed because act does not emulate those runners reliably.
EOF
}

validate_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --validate-only)
      validate_only=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

secret_args=()
if [[ -f "$SECRETS_FILE" ]]; then
  secret_args=(--secret-file "$SECRETS_FILE")
fi

common_act_args=(
  --container-architecture linux/amd64
  -P "ubuntu-latest=$ACT_IMAGE_UBUNTU"
  -P "ubuntu-22.04=$ACT_IMAGE_UBUNTU"
)

validate_workflow() {
  local workflow="$1"
  echo "==> validating $workflow"
  "$ACT_BIN" --validate -W "$workflow"
}

run_workflow() {
  local event_name="$1"
  local workflow="$2"
  shift 2

  echo "==> running $workflow ($event_name)"
  "$ACT_BIN" "$event_name" -W "$workflow" "${common_act_args[@]}" "${secret_args[@]}" "$@"
}

workflows=(
  .github/workflows/publish.yml
  .github/workflows/bundle-repro-check.yml
  .github/workflows/windows-binary-repro-investigation.yml
  .github/workflows/publish-smoke.yml
  .github/workflows/repro-env-check.yml
  .github/workflows/trivy.yml
  .github/workflows/rust-clippy.yml
  .github/workflows/rust-ci.yml
  .github/workflows/sonarcube.yml
  .github/workflows/covecode.yml
)

for workflow in "${workflows[@]}"; do
  validate_workflow "$workflow"
done

if [[ "$validate_only" -eq 1 ]]; then
  echo "Validation completed."
  exit 0
fi

run_workflow push .github/workflows/repro-env-check.yml
run_workflow push .github/workflows/rust-ci.yml
run_workflow push .github/workflows/rust-clippy.yml
run_workflow push .github/workflows/trivy.yml
run_workflow push .github/workflows/sonarcube.yml
run_workflow push .github/workflows/covecode.yml --env YARN_IGNORE_ENGINES=1
run_workflow workflow_dispatch .github/workflows/publish-smoke.yml --matrix platform:ubuntu-22.04

echo "Local act suite completed."
