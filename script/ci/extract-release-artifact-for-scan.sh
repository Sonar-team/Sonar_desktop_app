#!/usr/bin/env bash
set -euo pipefail

artifact="${1:?usage: extract-release-artifact-for-scan.sh <artifact> [scan-root]}"
scan_root="${2:-scan-root}"

if [[ ! -f "$artifact" ]]; then
  echo "Release artifact not found: ${artifact}" >&2
  exit 1
fi

artifact_abs="$(realpath "$artifact")"
artifact_name="$(basename "$artifact")"
artifact_lower="$(printf '%s' "$artifact_name" | tr '[:upper:]' '[:lower:]')"

rm -rf "$scan_root"
mkdir -p "$scan_root"

case "$artifact_lower" in
  *.deb)
    mkdir -p "${scan_root}/rootfs" "${scan_root}/control"
    dpkg-deb -x "$artifact_abs" "${scan_root}/rootfs"
    dpkg-deb -e "$artifact_abs" "${scan_root}/control"
    ;;
  *.rpm)
    mkdir -p "${scan_root}/rootfs" "${scan_root}/metadata"
    (
      cd "${scan_root}/rootfs"
      rpm2cpio "$artifact_abs" | cpio -idm --quiet
    )
    rpm -qip "$artifact_abs" > "${scan_root}/metadata/rpm-info.txt" 2>/dev/null || true
    ;;
  *.msi | *.exe)
    mkdir -p "${scan_root}/extracted"
    7z x -y "$artifact_abs" "-o${scan_root}/extracted"
    ;;
  *.dmg)
    mkdir -p "${scan_root}/extracted"
    if ! 7z x -y "$artifact_abs" "-o${scan_root}/extracted"; then
      echo "Unable to extract DMG with 7z; scanning raw artifact only" >&2
      cp "$artifact_abs" "${scan_root}/${artifact_name}"
    fi
    ;;
  *)
    echo "Unsupported artifact type; scanning raw artifact only: ${artifact_name}" >&2
    cp "$artifact_abs" "${scan_root}/${artifact_name}"
    ;;
esac

if ! find "$scan_root" -mindepth 1 -print -quit | grep -q .; then
  echo "No files extracted from ${artifact_name}" >&2
  exit 1
fi

printf '%s\n' "$scan_root"
