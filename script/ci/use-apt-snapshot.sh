#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSIONS_FILE="${VERSIONS_FILE:-${ROOT_DIR}/config/build-versions.env}"

if [[ ! -f "$VERSIONS_FILE" ]]; then
  echo "Build versions file not found: $VERSIONS_FILE" >&2
  exit 1
fi

set -a
# shellcheck source=/dev/null
source "$VERSIONS_FILE"
set +a

: "${APT_SNAPSHOT_TIMESTAMP:?APT_SNAPSHOT_TIMESTAMP is required}"
: "${DEBIAN_SNAPSHOT_BASE_URL:?DEBIAN_SNAPSHOT_BASE_URL is required}"
: "${UBUNTU_SNAPSHOT_BASE_URL:?UBUNTU_SNAPSHOT_BASE_URL is required}"

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "This script must run as root. Use sudo in CI when needed." >&2
    exit 1
  fi
}

detect_os() {
  if [[ ! -r /etc/os-release ]]; then
    echo "Unable to detect OS: /etc/os-release missing" >&2
    exit 1
  fi

  # shellcheck source=/dev/null
  source /etc/os-release
  printf '%s:%s\n' "${ID:-}" "${VERSION_CODENAME:-}"
}

write_debian_sources() {
  local codename="$1"
  local base="${DEBIAN_SNAPSHOT_BASE_URL}"
  local timestamp="${APT_SNAPSHOT_TIMESTAMP}"

  cat > /etc/apt/sources.list <<EOF
deb [check-valid-until=no] ${base}/debian/${timestamp} ${codename} main
deb [check-valid-until=no] ${base}/debian-security/${timestamp} ${codename}-security main
EOF
}

write_ubuntu_sources() {
  local codename="$1"
  local base="${UBUNTU_SNAPSHOT_BASE_URL}"
  local timestamp="${APT_SNAPSHOT_TIMESTAMP}"

  cat > /etc/apt/sources.list <<EOF
deb [check-valid-until=no] ${base}/${timestamp} ${codename} main universe
deb [check-valid-until=no] ${base}/${timestamp} ${codename}-updates main universe
deb [check-valid-until=no] ${base}/${timestamp} ${codename}-security main universe
EOF
}

main() {
  require_root

  local os_info os_id codename
  os_info="$(detect_os)"
  os_id="${os_info%%:*}"
  codename="${os_info#*:}"

  if [[ -z "$codename" ]]; then
    echo "Unable to detect OS codename" >&2
    exit 1
  fi

  mkdir -p /etc/apt/apt.conf.d
  printf '%s\n' 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99snapshot-no-check-valid-until

  case "$os_id" in
    debian)
      write_debian_sources "$codename"
      ;;
    ubuntu)
      write_ubuntu_sources "$codename"
      ;;
    *)
      echo "Unsupported apt snapshot OS: ${os_id}" >&2
      exit 1
      ;;
  esac

  apt-get -o Acquire::Check-Valid-Until=false update
}

main "$@"
