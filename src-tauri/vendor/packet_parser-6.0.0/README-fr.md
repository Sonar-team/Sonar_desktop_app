# packet_parser

[![CI](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Akmot9/Packet-parser/graph/badge.svg?token=5YpEN9abhE)](https://codecov.io/gh/Akmot9/Packet-parser)
[![Crates.io](https://img.shields.io/crates/v/packet_parser.svg)](https://crates.io/crates/packet_parser)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

`packet_parser` est une crate Rust de parsing de paquets reseau. Elle prend une
trame brute, commence a la couche liaison, puis remonte progressivement les
couches internet, transport et application.

Le coeur de l'API est `PacketFlow`: une representation empruntee, zero-copy, du
paquet parse. Les protocoles inconnus ou non supportes ne font pas echouer tout
le parsing: la crate conserve les couches deja decodees et laisse les couches
suivantes a `None` quand c'est necessaire.

![Packet parser overview](images/packet_parser.png)

## Installation

```toml
[dependencies]
packet_parser = "6.0.0"
```

Pour reproduire les exemples qui decodent de l'hexadecimal:

```toml
[dependencies]
hex = "0.4"
packet_parser = "6.0.0"
```

## Exemple rapide

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

## API principale

| Besoin | API |
| --- | --- |
| Parser une trame complete | `PacketFlow::try_from(&[u8])` |
| Parser seulement Ethernet/VLAN | `DataLink::try_from(&[u8])` |
| Parser seulement L3 | `Internet::try_from(&[u8])` |
| Parser seulement L4 | `Transport::try_from(&[u8])` ou `Transport::try_from_parts(...)` |
| Detacher le resultat du buffer d'origine | `flow.to_owned()` |
| Recuperer les flux encapsules | `flow.flatten()` |
| Mesurer le temps de parsing par couche | `PacketFlow::try_from_timed(...)` avec la feature `parse_timing` |

`PacketFlow` contient:

```rust
pub struct PacketFlow<'a> {
    pub data_link: DataLink<'a>,
    pub internet: Option<Internet<'a>>,
    pub transport: Option<Transport<'a>>,
    pub application: Option<Application>,
    pub inner: Option<Box<PacketFlow<'a>>>,
}
```

## Protocoles

### Liaison

- Ethernet II
- VLAN 802.1Q
- Adresses MAC et resolution OUI interne

### Internet

- ARP
- IPv4
- IPv6
- Profinet

Pour IPv4 fragmente, la crate ne fait pas de reassemblage IP. Dans ce cas,
`payload_protocol` vaut `None` pour eviter de parser une couche transport
incomplete.

### Transport

- TCP
- UDP
- Mapping de nombreux numeros de protocoles IP vers `TransportProtocol`

Les protocoles autres que TCP/UDP peuvent etre representes par leur enum, mais
ils ne fournissent pas toujours ports et payload applicatif.

### Application

La detection applicative est volontairement best-effort. Les modules de parsing
incluent notamment:

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

`PacketFlow` remonte actuellement un nom de protocole applicatif simple dans
`Application { application_protocol }`. Pour un parsing detaille d'un protocole
precis, utilisez directement le module correspondant dans
`packet_parser::parse::application::protocols`.

## Tunnels

`PacketFlow` peut representer plusieurs niveaux de flux via `inner`.

Le tunnel supporte aujourd'hui:

- CAPWAP-Data sur UDP/5247
- IEEE 802.11 encapsule
- LLC/SNAP vers la couche L3 interne

Exemple:

```rust
let flow = PacketFlow::try_from(packet.as_slice())?;

for level in flow.flatten() {
    println!("{:?} -> {:?}", level.internet, level.transport);
}
```

## Features

| Feature | Effet |
| --- | --- |
| `doc-diagrams` | Active les diagrammes Rustdoc via `aquamarine` |
| `parse_timing` | Expose `ParseTiming` et `PacketFlow::try_from_timed` |

La feature `parse_timing` est faite pour les benchmarks. Le chemin normal
`PacketFlow::try_from` ne mesure pas le temps de parsing.

Exemple:

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

Activation:

```bash
cargo test --features parse_timing
```

## Benchmarks et rapport HTML

Le harnais de benchmark principal est `tools/verbench`. Il compare les versions
publiees de la crate sur crates.io avec la copie locale, puis genere:

- `perf_by_version.json`
- `perf_by_version.html`

Execution complete:

```bash
tools/verbench/run.sh
```

Regenerer seulement le rapport HTML depuis le JSON existant:

```bash
python3 tools/verbench/report.py
```

Le rapport HTML est autonome: il s'ouvre directement dans le navigateur et ne
depend pas de Docker, Postgres, Grafana ou d'un CDN.

```bash
xdg-open perf_by_version.html
```

`tools/verbench` mesure les moyennes `l2_ns`, `l3_ns`, `l4_ns`, `l7_ns` et
`total_ns` sur un paquet de reference, apres warmup. Les chiffres servent a
comparer les tendances entre versions sur une meme machine, pas a publier une
latence absolue universelle.

## Pipeline PCAP optionnel

Le workspace contient aussi `benchmark_db`, un binaire qui parse des PCAP locaux
et ecrit des evenements JSONL avec:

- `run_id`
- `crate_code`
- `pcap`
- index du paquet
- hash du paquet
- duree totale
- timings OSI si `parse_timing` est active

Commande:

```bash
cargo run -p benchmark_db --release
```

Les fichiers sont ecrits dans:

```text
~/.local/share/packet_parser_bench/jsonl/
```

Le pipeline Docker `docker-compose.yml` peut ensuite ingester ces JSONL dans
Postgres et les afficher dans Grafana, mais il est optionnel.

## Exemples

Le dossier `examples/` contient plusieurs points d'entree utiles:

```bash
cargo run --example parse_tcp
cargo run --example parse_hex_dump
cargo run --example pars_quic
cargo run --example parse_pgadm
```

## Tests et qualite

Commandes courantes:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
cargo build --release
```

Pour les binaires qui lisent des PCAP via la crate `pcap`, installez aussi les
dependances systeme de libpcap. Sur Debian/Ubuntu:

```bash
sudo apt-get install libpcap-dev
```

## Limites connues

- Pas de reassemblage TCP.
- Pas de reassemblage IP.
- La detection applicative est heuristique et best-effort.
- Le chemin `parse_timing` est dedie aux mesures et ne doit pas etre confondu
  avec le chemin de parsing standard.
- Le parsing timé ne mesure pas encore recursivement les flux `inner` issus des
  tunnels.

## Licence

Distribue sous licence MIT. Voir [LICENSE.md](LICENSE.md).
