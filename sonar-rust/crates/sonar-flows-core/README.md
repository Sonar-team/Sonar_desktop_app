# sonar-flows-core

`sonar-flows-core` is the reusable domain library behind SONAR's desktop and
command-line applications. It turns parsed network packets and existing SONAR
Flow Matrix Standard (SFMS) files into deterministic flow matrices and network
graphs, without depending on Tauri.

The crate focuses on passive, offline data processing. Live capture, user
interfaces, and command-line argument handling belong to its consumers.

## Capabilities

| Module | Purpose |
| --- | --- |
| `matrix` | Aggregate packet metadata by flow, maintain labels and origins, and import or export SFMS rows. |
| `csv` | Strictly validate, read, reconstruct, and merge SFMS CSV files. |
| `graph` | Build a network graph and produce coalescible node and edge updates for a consumer. |
| `link` | Project typed link layers to the SFMS identity and reconstruct Ethernet, RAW, SLL, and SLL2 identities. |
| `packet` | Represent captured packets and flatten nested flows while retaining their tunnel relationship. |
| `pcap` | Optionally read PCAP/PCAPNG files and convert them to a flow matrix. |
| `error` | Provide the shared `Result` type and structured `SonarCoreError` errors. |

The default feature set is empty. Matrix, CSV, graph, and packet processing do
not require a native packet-capture library.

| Feature | Default | Effect |
| --- | --- | --- |
| `pcap` | No | Enables offline PCAP/PCAPNG reading through `libpcap` or Npcap. |

## Installation

From crates.io:

```toml
[dependencies]
sonar-flows-core = "0.2"
```

Enable offline capture-file conversion when needed:

```toml
[dependencies]
sonar-flows-core = { version = "0.2", features = ["pcap"] }
```

Within this repository, use the workspace dependency declared in
`sonar-rust/Cargo.toml`:

```toml
[dependencies]
sonar-flows-core.workspace = true
```

The package name uses hyphens; Rust imports it as `sonar_flows_core`.

## Merge SFMS matrices

This example uses only the default feature set:

```rust
use std::path::{Path, PathBuf};

fn main() -> sonar_flows_core::Result<()> {
    let inputs = vec![
        PathBuf::from("site-a.csv"),
        PathBuf::from("site-b.csv"),
    ];

    let rows = sonar_flows_core::csv::merge_matrix_files_to_csv(
        &inputs,
        Path::new("merged.csv"),
    )?;

    println!("exported {rows} flows");
    Ok(())
}
```

Every input is validated before it is merged. Counters are accumulated,
`last_seen` keeps the latest timestamp, labels are reapplied, and the `origin`
column records the source files associated with each flow.

## Convert PCAP files

Enable the `pcap` feature, then call the batch conversion API:

```rust
use std::path::{Path, PathBuf};

fn main() -> sonar_flows_core::Result<()> {
    let inputs = vec![PathBuf::from("capture.pcap")];

    let rows = sonar_flows_core::pcap::convert_pcap_files_to_csv(
        &inputs,
        Path::new("matrix.csv"),
        |path, report| {
            eprintln!(
                "{}: {} packets, {} parsed, {} unsupported",
                path.display(),
                report.packets,
                report.parse_ok,
                report.parse_errors,
            );
        },
    )?;

    println!("exported {rows} flows");
    Ok(())
}
```

Unsupported packets are counted and skipped. Opening or reading a corrupt or
truncated capture is a fatal error, so the batch APIs do not return a silently
partial matrix.

### Native dependency for `pcap`

The feature relies on the platform packet-capture library:

- Debian/Ubuntu: install `libpcap-dev`.
- Fedora: install `libpcap-devel`.
- macOS: use the system `libpcap`.
- Windows: install Npcap and configure its SDK/import library for the consuming
  build.

The packet types are supported on Linux, macOS, and Windows.

## SFMS output

An exported matrix contains one row per distinct flow:

```text
mac_source,mac_destination,vlan_id,protocol_data_link,ip_source,ip_source_type,label_source,ip_destination,ip_destination_type,label_destination,port_source,port_destination,protocol_transport,application_protocol,count,total_bytes,last_seen,encap_id,origin
```

Two extension columns preserve audit context:

- `encap_id` links external tunnel flows to their decapsulated child flows. A
  bare 16-character hexadecimal ID means that every packet in the row used one
  tunnel; `id:n|id:n` records per-tunnel packet counts.
- `origin` is empty for packets and PCAP imports. When matrices are merged, it
  contains the sorted, deduplicated source file names separated by `|`.

There is intentionally no `link_details` column. For Linux cooked captures,
the source address and carried protocol belong to the SFMS conversation
identity. Packet direction, ARPHRD type, declared address length, reserved
bits, and SLL2 interface index describe the observation point: packet events
retain them, but they neither split matrix rows nor prevent matrices from
different probes on the same network from merging. The `origin` column keeps
the contributing file names.
See [TUNNELS.md](../../../TUNNELS.md) for the tunnel model and its accounting
invariants.

## Data guarantees

- CSV imports reject malformed rows, invalid IP addresses, and invalid
  timestamps with the offending line number.
- CSV exports have deterministic row ordering and use atomic replacement to
  avoid leaving a truncated final file after an I/O failure.
- Labels that begin with spreadsheet formula characters are escaped on export
  and restored on re-import.
- Tunnel identifiers are deterministic across directions, builds, and Rust
  versions.
- CSV export and re-import preserve Ethernet, RAW, SLL, and SLL2 SFMS
  identities, per-tunnel packet accounting, labels, and flow origins.

Lower-level building blocks remain available when the batch helpers are too
coarse: `FlowMatrix`, `FlowMatrixRow`, `GraphData`, `GraphUpdateBatch`,
`CapturedPacketOwned`, and the functions in the `csv` and `pcap` modules.

## Development

The workspace requires Rust 1.97.1 and uses edition 2024. From `sonar-rust/`:

```sh
cargo fmt -- --check
cargo clippy -p sonar-flows-core --all-targets --all-features -- -D warnings
cargo test -p sonar-flows-core --all-features
cargo doc -p sonar-flows-core --all-features --no-deps
```

Install the native dependency above before building or testing with
`--all-features`.

The sibling [`sonar-flows-cli`](../sonar-flows-cli) crate is a complete batch
consumer of this library. For the broader workspace and product direction, see
the [Rust workspace README](../../README.md) and [SONAR vision](../../../VISION.md).

## License

Licensed under [AGPL-3.0-only](../../../LICENSE.md).
