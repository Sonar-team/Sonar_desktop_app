#!/usr/bin/env python3
"""Generate a directional flow oracle from fields shared by TShark and SFMS.

The output deliberately ignores application dissection. Values are numeric
where protocol naming differs between TShark and packet_parser.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path


FIELDS = (
    "frame.encap_type",
    "frame.time_epoch",
    "frame.len",
    "eth.src",
    "eth.dst",
    "eth.type",
    "eth.len",
    "vlan.id",
    "vlan.etype",
    "sll.src.eth",
    "sll.etype",
    "arp.src.proto_ipv4",
    "arp.dst.proto_ipv4",
    "ip.src",
    "ip.dst",
    "ip.len",
    "ipv6.src",
    "ipv6.dst",
    "ip.proto",
    "ipv6.nxt",
    "ipv6.hopopts.nxt",
    "ipv6.routing.nxt",
    "ipv6.dstopts.nxt",
    "ip.frag_offset",
    "ipv6.fraghdr.offset",
    "ipv6.fraghdr.nxt",
    "tcp.srcport",
    "tcp.dstport",
    "udp.srcport",
    "udp.dstport",
    "wlan.sa",
    "wlan.da",
    "llc.type",
)


@dataclass(frozen=True, order=True)
class FlowKey:
    mac_source: str
    mac_destination: str
    vlan_id: str
    ether_type: int
    ip_source: str
    ip_destination: str
    ip_protocol: str
    port_source: str
    port_destination: str


@dataclass
class FlowStats:
    count: int = 0
    total_bytes: int = 0
    last_seen_us: int = 0


def first(value: str) -> str:
    return value.split(",", 1)[0]


def last(value: str) -> str:
    return value.rsplit(",", 1)[-1]


def parse_hex(value: str) -> int:
    return int(value, 16)


def effective_ether_type(row: dict[str, str]) -> int:
    encapsulation = first(row["frame.encap_type"])
    if encapsulation in {"25", "210"}:  # Linux cooked v1/v2
        return parse_hex(first(row["sll.etype"]))
    if encapsulation == "7":  # LINKTYPE_RAW (TShark encap WTAP_ENCAP_RAW_IP)
        if first(row["ip.src"]):
            return 0x0800
        if first(row["ipv6.src"]):
            return 0x86DD
        return 0
    # packet_parser retains one VLAN tag and exposes its payload EtherType.
    vlan_payload = first(row["vlan.etype"])
    if first(row["vlan.id"]) and vlan_payload:
        return parse_hex(vlan_payload)
    ether_type = first(row["eth.type"])
    if ether_type:
        return parse_hex(ether_type)
    # IEEE 802.3 uses the same 16-bit slot as Ethernet II, as a length. SFMS
    # preserves that numeric value as an unknown protocol.
    return int(first(row["eth.len"]) or "0")


def packet_key(row: dict[str, str]) -> FlowKey | None:
    # SFMS v1 preserves one VLAN only. Stacked VLAN packets therefore have no
    # lossless common projection and are excluded explicitly.
    if "," in row["vlan.id"]:
        return None
    ether_type = effective_ether_type(row)
    # Values below 0x0600 are IEEE 802.3 lengths rather than EtherTypes. The
    # LLC/SNAP details needed to align both models are outside SFMS v1.
    if ether_type < 0x0600 or ether_type == 0x8100:
        return None
    ip_source = ""
    ip_destination = ""
    ip_protocol = ""
    port_source = ""
    port_destination = ""

    if ether_type == 0x0806:  # ARP: SFMS exposes protocol addresses.
        ip_source = first(row["arp.src.proto_ipv4"])
        ip_destination = first(row["arp.dst.proto_ipv4"])
    elif ether_type == 0x0800:  # Direct IPv4 only; do not pierce MPLS/PPPoE.
        ip_source = first(row["ip.src"])
        ip_destination = first(row["ip.dst"])
        candidate_protocol = first(row["ip.proto"])
        fragment_offset = int(first(row["ip.frag_offset"]) or "0")
        if fragment_offset == 0 and candidate_protocol == "6":
            port_source = first(row["tcp.srcport"])
            port_destination = first(row["tcp.dstport"])
            if port_source and port_destination:
                ip_protocol = candidate_protocol
        elif fragment_offset == 0 and candidate_protocol == "17":
            port_source = first(row["udp.srcport"])
            port_destination = first(row["udp.dstport"])
            if port_source and port_destination:
                ip_protocol = candidate_protocol
        elif fragment_offset == 0:
            ip_protocol = candidate_protocol
    elif ether_type == 0x86DD:  # Direct IPv6 only.
        ip_source = first(row["ipv6.src"])
        ip_destination = first(row["ipv6.dst"])
        extension_protocols = [
            first(row[field])
            for field in ("ipv6.hopopts.nxt", "ipv6.routing.nxt", "ipv6.dstopts.nxt")
            if first(row[field])
        ]
        candidate_protocol = extension_protocols[-1] if extension_protocols else first(row["ipv6.nxt"])
        has_fragment_header = bool(first(row["ipv6.fraghdr.nxt"]))
        if not has_fragment_header and candidate_protocol == "6":
            port_source = first(row["tcp.srcport"])
            port_destination = first(row["tcp.dstport"])
            if port_source and port_destination:
                ip_protocol = candidate_protocol
        elif not has_fragment_header and candidate_protocol == "17":
            port_source = first(row["udp.srcport"])
            port_destination = first(row["udp.dstport"])
            if port_source and port_destination:
                ip_protocol = candidate_protocol
        elif not has_fragment_header:
            ip_protocol = candidate_protocol

    encapsulation = first(row["frame.encap_type"])
    if encapsulation == "1":  # Ethernet
        mac_source = first(row["eth.src"])
        mac_destination = first(row["eth.dst"])
    elif encapsulation in {"25", "210"}:  # Linux cooked v1/v2
        mac_source = first(row["sll.src.eth"])
        mac_destination = ""
    elif encapsulation == "7":  # Raw IP
        mac_source = ""
        mac_destination = ""
    else:
        # The oracle is deliberately strict: a new encapsulation needs an
        # explicit mapping instead of silently being treated as Ethernet.
        return None

    return FlowKey(
        mac_source=mac_source,
        mac_destination=mac_destination,
        vlan_id=first(row["vlan.id"]),
        ether_type=ether_type,
        ip_source=ip_source,
        ip_destination=ip_destination,
        ip_protocol=ip_protocol,
        port_source=port_source,
        port_destination=port_destination,
    )


def capwap_inner_key(row: dict[str, str]) -> FlowKey | None:
    """Project the innermost 802.11/LLC flow exposed by TShark.

    This mode is intentionally CAPWAP-specific: it proves the tunnel rows and
    their byte accounting without pretending every TShark encapsulation uses
    the same repeated-field layout.
    """
    mac_source = first(row["wlan.sa"])
    mac_destination = first(row["wlan.da"])
    ether_type_text = last(row["llc.type"])
    if not mac_source or not mac_destination or not ether_type_text:
        return None
    ether_type = parse_hex(ether_type_text)
    ip_source = last(row["ip.src"] or row["ipv6.src"])
    ip_destination = last(row["ip.dst"] or row["ipv6.dst"])
    ip_protocol = last(row["ip.proto"] or row["ipv6.nxt"])
    port_source = last(row["tcp.srcport"] or row["udp.srcport"])
    port_destination = last(row["tcp.dstport"] or row["udp.dstport"])
    return FlowKey(
        mac_source=mac_source,
        mac_destination=mac_destination,
        vlan_id="",
        ether_type=ether_type,
        ip_source=ip_source,
        ip_destination=ip_destination,
        ip_protocol=ip_protocol,
        port_source=port_source,
        port_destination=port_destination,
    )


def tshark_rows(capture: Path) -> tuple[str, list[dict[str, str]]]:
    version = subprocess.run(
        ["tshark", "--version"], check=True, capture_output=True, text=True
    ).stdout.splitlines()[0]
    command = [
        "tshark",
        "-n",
        "-r",
        str(capture),
        "-T",
        "fields",
        "-E",
        "header=y",
        "-E",
        "separator=/t",
        "-E",
        "occurrence=a",
        "-E",
        "aggregator=,",
    ]
    for field in FIELDS:
        command.extend(("-e", field))
    output = subprocess.run(
        command, check=True, capture_output=True, text=True
    ).stdout.splitlines()
    return version, list(csv.DictReader(output, delimiter="\t"))


def generate(capture: Path, output: Path, capwap_inner: bool = False) -> None:
    version, packets = tshark_rows(capture)
    flows: dict[FlowKey, FlowStats] = defaultdict(FlowStats)
    included_packets = 0
    included_bytes = 0
    for packet in packets:
        key = capwap_inner_key(packet) if capwap_inner else packet_key(packet)
        if key is None:
            continue
        packet_bytes = (
            int(last(packet["ip.len"])) if capwap_inner else int(packet["frame.len"])
        )
        included_packets += 1
        included_bytes += packet_bytes
        stats = flows[key]
        stats.count += 1
        stats.total_bytes += packet_bytes
        timestamp_us = int(Decimal(packet["frame.time_epoch"]) * 1_000_000)
        stats.last_seen_us = max(stats.last_seen_us, timestamp_us)

    digest = hashlib.sha256(capture.read_bytes()).hexdigest()
    with output.open("w", encoding="utf-8", newline="") as target:
        target.write(f"# generator=script/pcap/generate-common-flows.py\n")
        target.write(f"# tshark={version}\n")
        target.write(f"# capture_sha256={digest}\n")
        target.write(f"# projection={'capwap_inner' if capwap_inner else 'outer'}\n")
        target.write(f"# included_packets={included_packets}\n")
        target.write(f"# included_bytes={included_bytes}\n")
        writer = csv.writer(target, delimiter="\t", lineterminator="\n")
        writer.writerow((*FlowKey.__dataclass_fields__, *FlowStats.__dataclass_fields__))
        for key, stats in sorted(flows.items()):
            writer.writerow((*key.__dict__.values(), *stats.__dict__.values()))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("capture", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--capwap-inner",
        action="store_true",
        help="project the innermost CAPWAP 802.11/LLC flow",
    )
    args = parser.parse_args()
    generate(args.capture, args.output, capwap_inner=args.capwap_inner)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
