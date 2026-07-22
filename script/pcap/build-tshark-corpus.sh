#!/usr/bin/env sh
set -eu

packet_parser_dir=${1:-../Packet-parser}
repo_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)
output_dir="$repo_dir/src-tauri/test_files/pcaps/import/pcap_tshark_corpus"
source_dir="$packet_parser_dir/pcaps_exemple"
temporary_dir=$(mktemp -d)
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

mkdir -p "$output_dir"
cp "$source_dir/sll.pcap" "$output_dir/linux_sll.pcap"
cp "$source_dir/capture_sll2.pcap" "$output_dir/linux_sll2.pcap"
cp "$source_dir/protocols/ieee80211/80211beacon.pcapng" \
    "$output_dir/unsupported_ieee80211.pcapng"

# Normalize the two useful Ethernet packets out of the heterogeneous merged
# PCAPNG. Their payload and original timestamps are preserved.
editcap -F pcap -T ether "$source_dir/capwap-association-valid.pcapng" \
    "$output_dir/capwap_management.pcap"

# Seven representative, losslessly modelled VLAN frames from Ultimate-PCAP.
editcap -r -F pcap -T ether "$source_dir/The-Ultimate-PCAP.pcapng" \
    "$output_dir/vlan.pcap" 2163 2166 2168 2176 2177 2184 2185

# Compact industrial/application slice of the large 4SICS capture.
editcap -r -F pcap -T ether "$source_dir/4SICS-GeekLounge-151020.pcap" \
    "$output_dir/industrial_ethernet.pcap" \
    1 2 30 31 907 909 1640 58528 145879 145889

# CAPWAP control/DTLS and RADIUS, kept separate from the plaintext data-plane
# fixture whose inner 802.11 flow is asserted below.
capwap_radius_source="$source_dir/vlan0--packet-capture CAPWAP a partir de ligne 520  (Ap = 192_168.0.104 ) + radius _ partir de la ligne 33003 .cap"
editcap -r -F pcap -T ether "$capwap_radius_source" \
    "$output_dir/capwap_radius.pcap" 520 521 527 528 607 608

# RAW IPv4 + IPv6 corpus derived only by removing the Ethernet header from
# real captures. -L adjusts both captured and on-wire lengths.
editcap -F pcap -T rawip -C 14 -L \
    "$source_dir/protocols/icmp/icmp_echo.pcapng" "$temporary_dir/raw4.pcap"
editcap -F pcap -T rawip -C 14 -L \
    "$source_dir/protocols/icmp/icmpv6_neighbor_solicitation.pcapng" \
    "$temporary_dir/raw6.pcap"
mergecap -a -F pcap -w "$output_dir/raw_ip.pcap" \
    "$temporary_dir/raw4.pcap" "$temporary_dir/raw6.pcap"

# The maintained crate embeds two complete real CAPWAP frames as its golden
# ToDS/FromDS tests. Rewrap those unchanged bytes in a deterministic PCAP.
perl -0777 -ne '
  $i=0;
  while (/fn sample_capwap_ieee80211_(?:inner_tcp|fromds_inner_tcp)\(\).*?hex::decode\(\s*"([0-9a-f]+)"/sg) {
    $i++;
    print "2000-01-01T00:00:0${i}.000000Z $1\n";
  }
' "$packet_parser_dir/src/parse/mod.rs" > "$temporary_dir/capwap.hex"
text2pcap -q -t ISO -r '^(?<time>[^ ]+) (?<data>[0-9a-f]+)$' \
    -F pcap -l 1 "$temporary_dir/capwap.hex" "$output_dir/capwap_data.pcap"

for name in linux_sll linux_sll2 raw_ip vlan industrial_ethernet \
    capwap_radius capwap_management capwap_data; do
    python3 "$repo_dir/script/pcap/generate-common-flows.py" \
        "$output_dir/$name.pcap" "$output_dir/$name.flows.tsv"
done
python3 "$repo_dir/script/pcap/generate-common-flows.py" --capwap-inner \
    "$output_dir/capwap_data.pcap" "$output_dir/capwap_data.inner.flows.tsv"
python3 "$repo_dir/script/pcap/generate-common-flows.py" \
    "$output_dir/unsupported_ieee80211.pcapng" \
    "$output_dir/unsupported_ieee80211.flows.tsv"
