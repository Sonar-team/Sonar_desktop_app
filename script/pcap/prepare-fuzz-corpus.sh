#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/../.." && pwd)"
source_dir="$repo_dir/src-tauri/test_files/pcaps/import/pcap_tshark_corpus"
corpus_dir="$repo_dir/sonar-rust/crates/sonar-flows-core/fuzz/corpus/pcap_reader"

mkdir -p -- "$corpus_dir"

# Petites captures couvrant Ethernet/VLAN, RAW, CAPWAP et la frontière DLT.
for capture_name in \
  vlan.pcap \
  raw_ip.pcap \
  industrial_ethernet.pcap \
  capwap_data.pcap \
  capwap_management.pcap \
  capwap_radius.pcap \
  unsupported_ieee80211.pcapng
do
  cp -- "$source_dir/$capture_name" "$corpus_dir/$capture_name"
done

# Deux entrées malformées déterministes : en-tête incomplet et dernier
# enregistrement tronqué. Elles partent d'une fixture dont l'oracle/hash est
# déjà contrôlé par pcap_accuracy, sans dupliquer de binaire dans Git.
head -c 12 -- "$source_dir/vlan.pcap" >"$corpus_dir/truncated_global_header.pcap"
capture_size="$(wc -c <"$source_dir/vlan.pcap")"
if (( capture_size <= 8 )); then
  echo "Fixture vlan.pcap trop petite pour produire une seed tronquée" >&2
  exit 1
fi
head -c "$((capture_size - 8))" -- "$source_dir/vlan.pcap" \
  >"$corpus_dir/truncated_packet.pcap"

printf 'Corpus fuzz PCAP préparé dans %s (%s seeds)\n' \
  "$corpus_dir" "$(find "$corpus_dir" -maxdepth 1 -type f | wc -l)"
