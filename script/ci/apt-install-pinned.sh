#!/usr/bin/env bash
set -euo pipefail

# Installe des paquets apt en résistant aux erreurs transitoires des miroirs
# (503 de snapshot.ubuntu.com) sans relâcher le pinning des versions.

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <paquet>..." >&2
  exit 1
fi

if [[ "${EUID}" -ne 0 ]]; then
  echo "This script must run as root. Use sudo in CI when needed." >&2
  exit 1
fi

APT_INSTALL_ATTEMPTS="${APT_INSTALL_ATTEMPTS:-5}"
APT_INSTALL_BACKOFF_SECONDS="${APT_INSTALL_BACKOFF_SECONDS:-15}"

apt_install() {
  apt-get \
    -o Acquire::Retries=5 \
    -o Acquire::http::Timeout=30 \
    -o Acquire::https::Timeout=30 \
    install -y --allow-downgrades "$@"
}

attempt=1
while true; do
  if apt_install "$@"; then
    exit 0
  fi
  if [[ "$attempt" -ge "$APT_INSTALL_ATTEMPTS" ]]; then
    echo "apt-get install failed after ${APT_INSTALL_ATTEMPTS} attempts" >&2
    exit 1
  fi
  echo "apt-get install failed (attempt ${attempt}/${APT_INSTALL_ATTEMPTS}); retrying in ${APT_INSTALL_BACKOFF_SECONDS}s" >&2
  sleep "$APT_INSTALL_BACKOFF_SECONDS"
  attempt=$((attempt + 1))
done
