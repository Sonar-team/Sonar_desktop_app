#!/usr/bin/env bash
set -euo pipefail

# Précharge le CRT MSVC et le SDK Windows dans une couche Docker persistante.
# cargo-xwin applique XWIN_HTTP_RETRIES pendant un build, mais sa commande
# `cache xwin` ne retente pas l'opération complète : cette boucle couvre aussi
# les coupures DNS/CDN qui interrompent le préchargement.

required_vars=(
  CARGO_XWIN_VERSION
  XWIN_VERSION
  XWIN_SDK_VERSION
  XWIN_CRT_VERSION
  XWIN_ARCH
  XWIN_HTTP_RETRIES
  XWIN_CACHE_DIR
)

for name in "${required_vars[@]}"; do
  if [[ -z "${!name:-}" ]]; then
    echo "Variable xwin requise absente : $name" >&2
    exit 1
  fi
done

if [[ ! "$XWIN_HTTP_RETRIES" =~ ^[0-9]+$ ]]; then
  echo "XWIN_HTTP_RETRIES doit être un entier positif ou nul" >&2
  exit 1
fi

if [[ "$XWIN_CACHE_DIR" != /* || "$XWIN_CACHE_DIR" == "/" ]]; then
  echo "XWIN_CACHE_DIR doit être un chemin absolu dédié" >&2
  exit 1
fi

attempt=0
while ! cargo xwin cache xwin; do
  if ((attempt >= XWIN_HTTP_RETRIES)); then
    echo "Échec du préchargement xwin après $((attempt + 1)) tentative(s)" >&2
    exit 1
  fi

  attempt=$((attempt + 1))
  delay=$((1 << (attempt - 1)))
  if ((delay > 30)); then
    delay=30
  fi

  # Même nettoyage ciblé que le chemin de retry interne de cargo-xwin. Le
  # splat déjà validé reste en place ; seuls les téléchargements/extractions
  # incomplets sont retirés.
  rm -rf -- \
    "${XWIN_CACHE_DIR%/}/xwin/dl" \
    "${XWIN_CACHE_DIR%/}/xwin/unpack"
  echo "Nouvelle tentative xwin $((attempt + 1))/$((XWIN_HTTP_RETRIES + 1)) dans ${delay}s" >&2
  sleep "$delay"
done

done_marker="${XWIN_CACHE_DIR%/}/xwin/DONE"
if [[ ! -s "$done_marker" ]]; then
  echo "Cache xwin incomplet : marqueur absent ($done_marker)" >&2
  exit 1
fi

metadata="${XWIN_CACHE_DIR%/}/SONAR_XWIN_TOOLCHAIN"
{
  printf 'CARGO_XWIN_VERSION=%s\n' "$CARGO_XWIN_VERSION"
  printf 'XWIN_VERSION=%s\n' "$XWIN_VERSION"
  printf 'XWIN_SDK_VERSION=%s\n' "$XWIN_SDK_VERSION"
  printf 'XWIN_CRT_VERSION=%s\n' "$XWIN_CRT_VERSION"
  printf 'XWIN_ARCH=%s\n' "$XWIN_ARCH"
} > "$metadata"

echo "Cache xwin prêt : $(tr '\n' ' ' < "$metadata")"
