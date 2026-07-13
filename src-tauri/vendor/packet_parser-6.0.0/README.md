# packet_parser

[![CI](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Akmot9/Packet-parser/graph/badge.svg?token=5YpEN9abhE)](https://codecov.io/gh/Akmot9/Packet-parser)
[![Crates.io](https://img.shields.io/crates/v/packet_parser.svg)](https://crates.io/crates/packet_parser)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

`packet_parser` is a Rust crate for parsing raw network packets. It starts at
the data-link layer and progressively decodes internet, transport and
application-layer information.

The main API is `PacketFlow`: a borrowed, zero-copy representation of a parsed
packet. Unknown or unsupported protocols do not make the whole parse fail. The
crate keeps the layers it could decode and leaves the next layers as `None`
when parsing cannot safely continue.

[Version francaise](README-fr.md)

![Packet parser overview](images/packet_parser.png)

## Installation

```toml
[dependencies]
packet_parser = "6.0.0"
```

For examples that decode hexadecimal packet dumps:

```toml
[dependencies]
hex = "0.4"
packet_parser = "6.0.0"
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

    println!("L2: {:?}", flow.data_link.ethertype);

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

## Main API

| Need | API |
| --- | --- |
| Parse a full Ethernet frame | `PacketFlow::try_from(&[u8])` |
| Parse only Ethernet/VLAN | `DataLink::try_from(&[u8])` |
| Parse only L3 | `Internet::try_from(&[u8])` |
| Parse only L4 | `Transport::try_from(&[u8])` or `Transport::try_from_parts(...)` |
| Detach the result from the original buffer | `flow.to_owned()` |
| Iterate over encapsulated flows | `flow.flatten()` |
| Measure parsing time per layer | `PacketFlow::try_from_timed(...)` with the `parse_timing` feature |

`PacketFlow` contains:

```rust
pub struct PacketFlow<'a> {
    pub data_link: DataLink<'a>,
    pub internet: Option<Internet<'a>>,
    pub transport: Option<Transport<'a>>,
    pub application: Option<Application>,
    pub inner: Option<Box<PacketFlow<'a>>>,
}
```

## Protocol Support

### Data Link

- Ethernet II
- VLAN 802.1Q
- MAC addresses and internal OUI resolution

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
| `parse_timing` | Exposes `ParseTiming` and `PacketFlow::try_from_timed` |

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
