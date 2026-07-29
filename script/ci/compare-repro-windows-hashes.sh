#!/usr/bin/env bash
set -euo pipefail

# Compare les manifests produits sur plusieurs runners du workflow
# repro-container. EXPECTED_WINDOWS_SHA256 peut contenir le hash obtenu sur une
# machine locale au même commit pour étendre la preuve CI à local <-> CI.

HASH_ROOT="${1:-repro-hashes-windows-cross}"
EXPECTED_WINDOWS_SHA256="${EXPECTED_WINDOWS_SHA256:-}"

if [[ ! -d "$HASH_ROOT" ]]; then
  echo "Dossier de hashes introuvable : $HASH_ROOT" >&2
  exit 1
fi

manifests=()
for machine in machine-a machine-b; do
  machine_root="${HASH_ROOT%/}/repro-hashes-windows-cross-${machine}"
  if [[ ! -d "$machine_root" ]]; then
    echo "Artifact Windows du runner ${machine} introuvable : $machine_root" >&2
    exit 1
  fi

  mapfile -d '' machine_manifests < <(
    find "$machine_root" -type f -name SHA256SUMS -print0 | sort -z
  )
  if ((${#machine_manifests[@]} != 2)); then
    echo "Le runner ${machine} doit fournir exactement deux manifests SHA256SUMS" >&2
    exit 1
  fi
  manifests+=("${machine_manifests[@]}")
done

if ((${#manifests[@]} != 4)); then
  echo "Quatre manifests issus de deux runners distincts sont requis" >&2
  exit 1
fi

reference_hash=""
for manifest in "${manifests[@]}"; do
  binary_hash="$(awk '$2 == "./windows/sonar.exe" { print $1 }' "$manifest")"
  if [[ ! "$binary_hash" =~ ^[[:xdigit:]]{64}$ ]]; then
    echo "Hash sonar.exe absent ou invalide dans $manifest" >&2
    exit 1
  fi
  binary_hash="${binary_hash,,}"
  echo "$manifest : $binary_hash"

  if [[ -z "$reference_hash" ]]; then
    reference_hash="$binary_hash"
  elif [[ "$binary_hash" != "$reference_hash" ]]; then
    echo "ÉCHEC : divergence Windows cross entre runners" >&2
    exit 1
  fi
done

if [[ -n "$EXPECTED_WINDOWS_SHA256" ]]; then
  if [[ ! "$EXPECTED_WINDOWS_SHA256" =~ ^[[:xdigit:]]{64}$ ]]; then
    echo "EXPECTED_WINDOWS_SHA256 doit contenir exactement 64 caractères hexadécimaux" >&2
    exit 1
  fi
  expected="${EXPECTED_WINDOWS_SHA256,,}"
  if [[ "$reference_hash" != "$expected" ]]; then
    echo "ÉCHEC : CI=$reference_hash, machine locale=$expected" >&2
    exit 1
  fi
  echo "OK : sonar.exe identique entre les runners CI et la machine locale ($reference_hash)"
else
  echo "OK : sonar.exe identique entre les runners CI ($reference_hash)"
  echo "Note : fournir windows_reference_sha256 au prochain dispatch pour prouver local <-> CI."
fi
