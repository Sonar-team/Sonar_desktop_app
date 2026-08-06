#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/../.." && pwd)"
source_dir="$repo_dir/src-tauri/test_files/pcaps/import/pcap_tshark_corpus"
fuzz_dir="$repo_dir/sonar-rust/crates/sonar-flows-core/fuzz"
corpus_dir="$fuzz_dir/corpus/pcap_reader"
matrix_corpus_dir="$fuzz_dir/corpus/matrix_reader"

mkdir -p -- "$corpus_dir" "$matrix_corpus_dir"

# Petites captures couvrant Ethernet/VLAN, RAW, CAPWAP et la frontière DLT.
for capture_name in \
  vlan.pcap \
  raw_ip.pcap \
  industrial_ethernet.pcap \
  capwap_data.pcap \
  capwap_management.pcap \
  capwap_radius.pcap \
  unsupported_ieee80211.pcapng \
  loopback_null.pcap
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

# Seeds de régression promues en fixtures versionnées (#151) : chaque crash
# corrigé reste dans le corpus pour toujours.
for target in pcap_reader matrix_reader; do
  if compgen -G "$fuzz_dir/regressions/$target/*" >/dev/null; then
    cp -- "$fuzz_dir/regressions/$target"/* "$fuzz_dir/corpus/$target/"
  fi
done

# Corpus du lecteur de matrice : la fixture SFMS de référence et des
# variantes déterministes (préambule v2 avec contexte percent-encodé,
# en-tête seul, préambule coupé en plein échappement).
matrix_fixture="$repo_dir/src-tauri/test_files/20260703_NP_Matrice.csv"
cp -- "$matrix_fixture" "$matrix_corpus_dir/reference.csv"
head -n 2 -- "$matrix_fixture" >"$matrix_corpus_dir/header_only.csv"
printf '#SFMS version=2 dlt=ETHERNET site=Usine%%20Nord sensor=sonde%%3DA interface=eth0\n' \
  >"$matrix_corpus_dir/v2_context_preamble.csv"
printf '#SFMS version=2 dlt=ETHERNET site=Usine%%2' \
  >"$matrix_corpus_dir/preamble_cut_mid_escape.csv"

printf 'Corpus fuzz préparé : %s seeds pcap_reader, %s seeds matrix_reader\n' \
  "$(find "$corpus_dir" -maxdepth 1 -type f | wc -l)" \
  "$(find "$matrix_corpus_dir" -maxdepth 1 -type f | wc -l)"
