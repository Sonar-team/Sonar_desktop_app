# packet_parser

[![CI](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Akmot9/Packet-parser/graph/badge.svg?token=5YpEN9abhE)](https://codecov.io/gh/Akmot9/Packet-parser)
[![Crates.io](https://img.shields.io/crates/v/packet_parser.svg)](https://crates.io/crates/packet_parser)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

`packet_parser` is a Rust crate for parsing raw network packets. It starts at
the data-link layer and progressively decodes internet, transport and
application-layer information.

The main API is `PacketFlow`: a borrowed, zero-copy representation of a parsed
packet. Unknown or unsupported protocols above the data-link layer do not make
the whole parse fail. The crate keeps the layers it could decode and leaves the
next layers as `None` when parsing cannot safely continue.

[Version francaise](README-fr.md)

![Packet parser overview](images/packet_parser.png)

## Installation

```toml
[dependencies]
packet_parser = "7.0.0"
```

For examples that decode hexadecimal packet dumps:

```toml
[dependencies]
hex = "0.4"
packet_parser = "7.0.0"
```

## Quick Example

```rust
use packet_parser::PacketFlow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = hex::decode(
        "feaa81e86d1efeaa818ec864080045500034000000003d06206b36e6700d\
         ac140a0201bbc1087d7f02aa4e2b998e80100081748300000101080a9373\
         c9c207ef14e3",
    )?;

    let flow = PacketFlow::try_from(raw.as_slice())?;

    println!("L2: {}", flow.data_link);

    if let Some(internet) = &flow.internet {
        println!(
            "L3: {} {:?} -> {:?}",
            internet.protocol_name, internet.source, internet.destination
        );
    }

    if let Some(transport) = &flow.transport {
        println!(
            "L4: {:?} {:?} -> {:?}",
            transport.protocol, transport.source_port, transport.destination_port
        );
    }

    if let Some(application) = &flow.application {
        println!("L7: {}", application.application_protocol);
    }

    Ok(())
}
```

This example uses the Ethernet compatibility API available in the published
7.0.0 release.

## Explicit LINKTYPE API (unreleased, target 7.0.0)

The development branch introduces an explicit, fail-closed entry point for
capture readers:

```rust
use packet_parser::{LinkType, is_supported, parse};

let link_type = LinkType::ETHERNET;
if !is_supported(link_type) {
    return Err(format!("unsupported LINKTYPE {}", link_type).into());
}

let flow = parse(link_type, packet_bytes)?;
```

`packet_bytes` must contain exactly one packet, without a PCAP or PCAPNG record
header. `LinkType` uses the canonical `LINKTYPE_*` namespace stored in capture
files. A live-capture adapter must normalize `DLT_*` values first when their
numeric value differs. For PCAPNG, the capture reader resolves the interface
referenced by each packet and passes that interface's LINKTYPE.

Current status on the development branch:

| LINKTYPE | Value | Decoder status |
| --- | ---: | --- |
| Ethernet | 1 | Supported |
| RAW IP | 101 | Supported for IPv4 and IPv6 |
| Native IEEE 802.11 | 105 | Modelled for CAPWAP inner flows; top-level decoder not yet supported |
| Linux SLL v1 | 113 | Supported |
| Bluetooth H4 with pseudo-header | 201 | Identified, explicitly unsupported |
| Linux SLL v2 | 276 | Supported |
| Any other value | Preserved as-is | `ParseError::UnsupportedLinkType` |

An unsupported LINKTYPE is rejected before packet bytes are decoded. Unknown
upper-layer protocols still use the graceful `None`/`corrupted` behaviour
described above.

For RAW IP, an empty packet or a version nibble other than 4/6 returns a
structured `InvalidLinkLayer(LinkLayerError)`. Once IPv4 or IPv6 is identified,
an invalid or truncated IP header remains a successful partial flow with
`corrupted: Internet`; the link layer and its accounting are preserved.

Linux SLL v1 decodes its 16-byte cooked header in network byte order and keeps
the packet type, raw ARPHRD hardware type, declared address length, available
source-address bytes and protocol value. Unknown numeric values are preserved;
an address longer than the eight-byte wire slot is reported as truncated rather
than rejected. Use canonical `LinkType::LINUX_SLL` (113): the value 25 displayed
by some Wireshark fields is an internal WTAP encapsulation identifier.

Linux SLL v2 independently decodes its 20-byte header and additionally keeps
the numeric capture-machine interface index and the reserved-MBZ field. A
non-zero reserved value is preserved and reported by `reserved_is_zero()`
rather than discarding an otherwise decodable packet, matching Tshark's
tolerant dissection. Interface names are not resolved because they belong to
the capture machine. Use canonical `LinkType::LINUX_SLL2` (276); Wireshark's
current internal WTAP encapsulation identifier for this format is 210.

Every parsed flow now carries a generic `LinkLayer`. Its common accessors do
not assume Ethernet:

```rust
println!("LINKTYPE={}", flow.data_link.link_type());
println!("next={:?}", flow.data_link.network_protocol());

if let Some(ethernet) = flow.data_link.as_ethernet() {
    println!("{} -> {}", ethernet.source_mac, ethernet.destination_mac);
}
```

`network_payload()` returns the borrowed L3 slice. Format-specific views are
explicit (`as_ethernet()`, `as_raw_ip()`, `as_linux_sll()`,
`as_linux_sll2()`, `as_ieee80211()`), so RAW and both SLL formats cannot
silently manufacture Ethernet fields.

## Main API

| Need | API |
| --- | --- |
| Check whether a link decoder exists (target 7.0.0) | `is_supported(LinkType)` |
| Parse a packet with an explicit link type (target 7.0.0) | `parse(LinkType, &[u8])` |
| Parse Ethernet with the compatibility shortcut | `PacketFlow::try_from(&[u8])` |
| Parse only Ethernet/VLAN | `DataLink::try_from(&[u8])` |
| Parse only L3 | `Internet::try_from(&[u8])` |
| Parse only L4 | `Transport::try_from(&[u8])` or `Transport::try_from_parts(...)` |
| Detach the result from the original buffer | `flow.to_owned()` |
| Iterate over encapsulated flows | `flow.flatten()` |
| Measure an explicit LINKTYPE (target 7.0.0) | `parse_timed(...)` with the `parse_timing` feature |
| Measure Ethernet through the compatibility API | `PacketFlow::try_from_timed(...)` with the `parse_timing` feature |

`PacketFlow` contains:

```rust
pub struct PacketFlow<'a> {
    pub data_link: LinkLayer<'a>,
    pub internet: Option<Internet<'a>>,
    pub transport: Option<Transport<'a>>,
    pub application: Option<Application>,
    pub inner: Option<Box<PacketFlow<'a>>>,
}
```

The 7.0 serialization schema nests the link layer and uses stable tags. The
borrowed and owned link models serialize identically (payload bytes are not
serialized):

```json
{
  "data_link": {
    "link_type": 1,
    "network_protocol": { "kind": "ipv4" },
    "link_kind": "ethernet",
    "link_details": {
      "destination_mac": "00:11:22:33:44:55",
      "source_mac": "66:77:88:99:aa:bb",
      "ethertype": "IPv4"
    }
  }
}
```

## Protocol Support

### Data Link

- Ethernet II
- VLAN 802.1Q
- RAW IPv4/IPv6 (`LINKTYPE_RAW`)
- Linux cooked capture v1 (`LINKTYPE_LINUX_SLL`)
- Linux cooked capture v2 (`LINKTYPE_LINUX_SLL2`)
- MAC addresses and internal OUI resolution
- Native IEEE 802.11 representation for CAPWAP inner flows (not yet a
  top-level LINKTYPE decoder)

### Internet

- ARP
- IPv4
- IPv6
- Profinet

For fragmented IPv4 packets, the crate does not perform IP reassembly. In that
case `payload_protocol` is set to `None` so the transport layer is not parsed
from incomplete data.

### Transport

- TCP
- UDP
- Mapping from many IP protocol numbers to `TransportProtocol`

Protocols other than TCP/UDP can be represented by the enum, but they do not
always expose ports or application payloads.

### Application

Application detection is intentionally best-effort. Parser modules include:

- DNS
- TLS
- SNMP
- NTP
- DHCP / DHCPv6
- HTTP
- MQTT
- PostgreSQL
- Modbus TCP
- EtherNet/IP
- OPC UA
- S7Comm
- COTP
- AMS
- GIOP
- SRVLOC
- QUIC
- Bitcoin

`PacketFlow` currently exposes a lightweight application protocol name through
`Application { application_protocol }`. For detailed protocol-specific parsing,
use the corresponding module under
`packet_parser::parse::application::protocols`.

## Tunnels

`PacketFlow` can represent several flow levels through `inner`.

The currently supported tunnel path is:

- CAPWAP-Data over UDP/5247
- Encapsulated IEEE 802.11
- LLC/SNAP to the inner L3 packet

Example:

```rust
let flow = PacketFlow::try_from(packet.as_slice())?;

for level in flow.flatten() {
    println!("{:?} -> {:?}", level.internet, level.transport);
}
```

## Features

| Feature | Effect |
| --- | --- |
| `doc-diagrams` | Enables Rustdoc diagrams through `aquamarine` |
| `parse_timing` | Exposes `ParseTiming`, `PacketFlow::try_from_timed` and, on the development branch, `parse_timed` |

The `parse_timing` feature is intended for benchmarks. The normal
`PacketFlow::try_from` path does not measure parsing time.

Example:

```rust
use packet_parser::{PacketFlow, timing::ParseTiming};

let mut timing = ParseTiming::default();
let flow = PacketFlow::try_from_timed(packet.as_slice(), &mut timing)?;

println!("L2={}ns L3={}ns L4={}ns L7={}ns total={}ns",
    timing.l2_ns,
    timing.l3_ns,
    timing.l4_ns,
    timing.l7_ns,
    timing.total_ns,
);
```

Enable it with:

```bash
cargo test --features parse_timing
```

## Benchmarks and HTML Report

The main benchmark harness is `tools/verbench`. It compares published crate
versions from crates.io with the local working copy, then generates:

- `perf_by_version.json`
- `perf_by_version.html`

Run the full benchmark:

```bash
tools/verbench/run.sh
```

Regenerate only the HTML report from an existing JSON file:

```bash
python3 tools/verbench/report.py
```

The HTML report is standalone. It opens directly in a browser and does not need
Docker, Postgres, Grafana or any CDN.

```bash
xdg-open perf_by_version.html
```

`tools/verbench` reports average `l2_ns`, `l3_ns`, `l4_ns`, `l7_ns` and
`total_ns` values on a fixed reference packet after warmup. Use these numbers to
compare trends between versions on the same machine, not as universal absolute
latency claims.

## Optional PCAP Pipeline

The workspace also contains `benchmark_db`, a binary that parses local PCAP
files and writes JSONL events containing:

- `run_id`
- `crate_code`
- `pcap`
- packet index
- packet hash
- total duration
- OSI timings when `parse_timing` is enabled

Run it with:

```bash
cargo run -p benchmark_db --release
```

Output files are written to:

```text
~/.local/share/packet_parser_bench/jsonl/
```

The optional `docker-compose.yml` pipeline can ingest those JSONL files into
Postgres and display them in Grafana. This is not required for the standalone
`verbench` HTML report.

## Examples

The `examples/` directory contains several useful entry points:

```bash
cargo run --example parse_tcp
cargo run --example parse_hex_dump
cargo run --example pars_quic
cargo run --example parse_pgadm
```

## Tests and Quality

Common checks:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
cargo build --release
```

For binaries that read PCAP files through the `pcap` crate, install the system
libpcap development package. On Debian/Ubuntu:

```bash
sudo apt-get install libpcap-dev
```

## Known Limitations

- No TCP reassembly.
- No IP reassembly.
- Application detection is heuristic and best-effort.
- The `parse_timing` path is dedicated to measurement and should not be treated
  as the standard parsing path.
- Timed parsing does not yet recursively measure `inner` flows produced by
  tunnel parsing.

## License

Distributed under the MIT license. See [LICENSE.md](LICENSE.md).
