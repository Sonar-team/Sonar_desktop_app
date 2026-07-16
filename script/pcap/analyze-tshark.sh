#!/usr/bin/env bash
set -euo pipefail

export LC_ALL=C
export TZ=UTC

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat >&2 <<'EOF'
Usage: analyze-tshark.sh <capture> [output.md] [expected.csv]

Generates a reproducible TShark report. By default, the report and optional
expected CSV are looked up next to the capture using the capture stem.
EOF
}

capture_path="${1:-}"
if [[ -z "$capture_path" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$capture_path" ]]; then
  echo "Capture not found: $capture_path" >&2
  exit 1
fi

for command_name in tshark capinfos awk mktemp sed; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command not found: $command_name" >&2
    exit 1
  fi
done

capture_dir="$(cd "$(dirname "$capture_path")" && pwd)"
capture_name="$(basename "$capture_path")"
capture_stem="${capture_name%.*}"
capture_path="${capture_dir}/${capture_name}"
output_path="${2:-${capture_dir}/${capture_stem}.tshark.md}"
csv_path="${3:-${capture_dir}/${capture_stem}.csv}"

output_dir="$(dirname "$output_path")"
if [[ ! -d "$output_dir" ]]; then
  echo "Output directory not found: $output_dir" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  hash_command=(sha256sum)
elif command -v shasum >/dev/null 2>&1; then
  hash_command=(shasum -a 256)
else
  echo "Required command not found: sha256sum or shasum" >&2
  exit 1
fi

tshark_version="$(tshark --version | sed -n '1p')"
capture_hash="$("${hash_command[@]}" "$capture_path" | awk '{print $1}')"

read -r pcap_packets pcap_bytes <<< "$(
  tshark -r "$capture_path" -T fields -e frame.number -e frame.len |
    awk -F '\t' '{ packets += 1; bytes += $2 } END { printf "%d %d\n", packets, bytes }'
)"

csv_summary=""
csv_status=""
if [[ -f "$csv_path" ]]; then
  if ! command -v python3 >/dev/null 2>&1; then
    echo "Required command not found for CSV verification: python3" >&2
    exit 1
  fi

  csv_summary="$(python3 "$SCRIPT_DIR/summarize-matrix-csv.py" "$csv_path")"

  read -r csv_rows csv_packets csv_bytes <<< "$csv_summary"
  if [[ "$pcap_packets" == "$csv_packets" && "$pcap_bytes" == "$csv_bytes" ]]; then
    csv_status="PASS"
  else
    csv_status="FAIL"
  fi
fi

tmp_report="$(mktemp "${output_path}.tmp.XXXXXX")"
cleanup() {
  rm -f "$tmp_report"
}
trap cleanup EXIT

{
  printf '# TShark analysis: `%s`\n\n' "$capture_name"
  printf -- '- Generator: `script/pcap/analyze-tshark.sh`\n'
  printf -- '- TShark: `%s`\n' "$tshark_version"
  printf -- '- Time zone: `UTC`\n'
  printf -- '- SHA-256: `%s`\n' "$capture_hash"

  printf '\n## Aggregate verification\n\n'
  printf '| Source | Rows/frames | Packet count | Total bytes | Result |\n'
  printf '|---|---:|---:|---:|---|\n'
  printf '| PCAP | %s | %s | %s | - |\n' "$pcap_packets" "$pcap_packets" "$pcap_bytes"
  if [[ -n "$csv_summary" ]]; then
    printf '| `%s` | %s | %s | %s | **%s** |\n' \
      "$(basename "$csv_path")" "$csv_rows" "$csv_packets" "$csv_bytes" "$csv_status"
  else
    printf '| Expected CSV | - | - | - | Not found |\n'
  fi

  printf '\n## Capture metadata\n\n```text\n'
  (cd "$capture_dir" && capinfos "$capture_name")
  printf '```\n'

  printf '\n## Protocol hierarchy\n\n```text\n'
  tshark -n -q -r "$capture_path" -z io,phs
  printf '```\n'

  printf '\n## Packet inventory\n\n```text\n'
  tshark -n -r "$capture_path" \
    -T fields -E header=y -E separator='|' -E quote=d \
    -e frame.number \
    -e frame.time_relative \
    -e frame.len \
    -e _ws.col.Protocol \
    -e eth.src \
    -e eth.dst \
    -e arp.opcode \
    -e arp.src.proto_ipv4 \
    -e arp.dst.proto_ipv4 \
    -e ip.src \
    -e ip.dst \
    -e tcp.srcport \
    -e tcp.dstport \
    -e tcp.flags.str \
    -e tcp.len \
    -e _ws.col.Info
  printf '```\n'

  printf '\n## Conversations\n\n```text\n'
  tshark -n -q -r "$capture_path" -z conv,eth -z conv,ip -z conv,tcp
  printf '```\n'

  printf '\n## Endpoints\n\n```text\n'
  tshark -n -q -r "$capture_path" -z endpoints,eth -z endpoints,ip -z endpoints,tcp
  printf '```\n'

  printf '\n## TLS records\n\n```text\n'
  tshark -n -r "$capture_path" -Y tls \
    -T fields -E header=y -E separator='|' -E quote=d \
    -e frame.number \
    -e ip.src \
    -e tcp.srcport \
    -e ip.dst \
    -e tcp.dstport \
    -e tls.record.content_type \
    -e tls.record.version \
    -e tls.record.length \
    -e _ws.col.Info
  printf '```\n'

  printf '\n## Expert information\n\n```text\n'
  tshark -n -q -r "$capture_path" -z expert
  printf '```\n'
} | sed 's/[[:space:]]*$//' > "$tmp_report"

mv "$tmp_report" "$output_path"
trap - EXIT

echo "Wrote $output_path"

if [[ "$csv_status" == "FAIL" ]]; then
  echo "PCAP and CSV aggregate totals do not match" >&2
  exit 1
fi
