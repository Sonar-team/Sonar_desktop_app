# packet_parser

[![CI](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/Akmot9/Packet-parser/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Akmot9/Packet-parser/graph/badge.svg?token=5YpEN9abhE)](https://codecov.io/gh/Akmot9/Packet-parser)
[![Crates.io](https://img.shields.io/crates/v/packet_parser.svg)](https://crates.io/crates/packet_parser)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

`packet_parser` est une crate Rust de parsing de paquets reseau. Elle prend une
trame brute, commence a la couche liaison, puis remonte progressivement les
couches internet, transport et application.

Le coeur de l'API est `PacketFlow`: une representation empruntee, zero-copy, du
paquet parse. Les protocoles inconnus ou non supportes au-dessus de la couche
liaison ne font pas echouer tout le parsing: la crate conserve les couches deja
decodees et laisse les couches suivantes a `None` quand c'est necessaire.

![Packet parser overview](images/packet_parser.png)

## Installation

```toml
[dependencies]
packet_parser = "7.0.0"
```

Pour reproduire les exemples qui decodent de l'hexadecimal:

```toml
[dependencies]
hex = "0.4"
packet_parser = "7.0.0"
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

Cet exemple utilise l'API de compatibilite Ethernet disponible dans la version
7.0.0 publiee.

## API LINKTYPE explicite (non publiee, cible 7.0.0)

La branche de developpement introduit une entree explicite et fermee par defaut
pour les lecteurs de captures:

```rust
use packet_parser::{LinkType, is_supported, parse};

let link_type = LinkType::ETHERNET;
if !is_supported(link_type) {
    return Err(format!("LINKTYPE non supporte: {}", link_type).into());
}

let flow = parse(link_type, packet_bytes)?;
```

`packet_bytes` doit contenir exactement un paquet, sans en-tete d'enregistrement
PCAP ou PCAPNG. `LinkType` utilise l'espace canonique `LINKTYPE_*` stocke dans
les fichiers. Un adaptateur de capture live doit d'abord normaliser les valeurs
`DLT_*` lorsque leur valeur numerique differe. Pour PCAPNG, le lecteur resout
l'interface referencee par chaque paquet et transmet le LINKTYPE de cette
interface.

Etat actuel de la branche de developpement:

| LINKTYPE | Valeur | Etat du decodeur |
| --- | ---: | --- |
| Ethernet | 1 | Supporte |
| RAW IP | 101 | Supporte pour IPv4 et IPv6 |
| IEEE 802.11 natif | 105 | Modelise pour les flux internes CAPWAP ; decodeur de capture non disponible |
| Linux SLL v1 | 113 | Supporte |
| Bluetooth H4 avec pseudo-en-tete | 201 | Identifie, explicitement non supporte |
| Linux SLL v2 | 276 | Supporte |
| Toute autre valeur | Preservee telle quelle | `ParseError::UnsupportedLinkType` |

Un LINKTYPE non supporte est refuse avant de decoder les octets du paquet. Les
protocoles inconnus des couches superieures conservent le comportement gracieux
`None`/`corrupted` decrit plus haut.

Pour RAW IP, un paquet vide ou un nibble de version different de 4/6 retourne
un `InvalidLinkLayer(LinkLayerError)` structure. Des qu'IPv4 ou IPv6 est
identifie, un header IP invalide ou tronque reste un flux partiel reussi avec
`corrupted: Internet` ; la liaison et sa comptabilite sont conservees.

Linux SLL v1 decode son en-tete cooked de 16 octets en ordre reseau et conserve
le type de paquet, le type materiel ARPHRD brut, la longueur d'adresse declaree,
les octets d'adresse source disponibles et la valeur du protocole. Les valeurs
numeriques inconnues sont preservees ; une adresse plus longue que le slot wire
de huit octets est signalee tronquee plutot que rejetee. Utiliser le
`LinkType::LINUX_SLL` canonique (113) : la valeur 25 affichee par certains
champs Wireshark est un identifiant d'encapsulation WTAP interne.

Linux SLL v2 decode independamment son en-tete de 20 octets et conserve en plus
l'index numerique de l'interface de la machine de capture ainsi que le champ
reserve MBZ. Une valeur reservee non nulle est preservee et signalee par
`reserved_is_zero()` plutot que de perdre un paquet autrement decodable, comme
le fait le dissecteur tolerant de Tshark. Les noms d'interface ne sont pas
resolus, car ils appartiennent a la machine de capture. Utiliser le
`LinkType::LINUX_SLL2` canonique (276) ; l'identifiant d'encapsulation WTAP
interne actuellement affiche par Wireshark pour ce format vaut 210.

Chaque flux parse transporte maintenant un `LinkLayer` generique. Ses
accesseurs communs ne supposent pas Ethernet:

```rust
println!("LINKTYPE={}", flow.data_link.link_type());
println!("suivant={:?}", flow.data_link.network_protocol());

if let Some(ethernet) = flow.data_link.as_ethernet() {
    println!("{} -> {}", ethernet.source_mac, ethernet.destination_mac);
}
```

`network_payload()` retourne le slice L3 emprunte. Les vues propres au format
sont explicites (`as_ethernet()`, `as_raw_ip()`, `as_linux_sll()`,
`as_linux_sll2()`, `as_ieee80211()`), afin que RAW et les deux formats SLL ne
puissent jamais fabriquer silencieusement des champs Ethernet.

## API principale

| Besoin | API |
| --- | --- |
| Verifier la presence d'un decodeur (cible 7.0.0) | `is_supported(LinkType)` |
| Parser un paquet avec un type de liaison explicite (cible 7.0.0) | `parse(LinkType, &[u8])` |
| Parser Ethernet avec le raccourci de compatibilite | `PacketFlow::try_from(&[u8])` |
| Parser seulement Ethernet/VLAN | `DataLink::try_from(&[u8])` |
| Parser seulement L3 | `Internet::try_from(&[u8])` |
| Parser seulement L4 | `Transport::try_from(&[u8])` ou `Transport::try_from_parts(...)` |
| Detacher le resultat du buffer d'origine | `flow.to_owned()` |
| Recuperer les flux encapsules | `flow.flatten()` |
| Mesurer un LINKTYPE explicite (cible 7.0.0) | `parse_timed(...)` avec la feature `parse_timing` |
| Mesurer Ethernet via l'API de compatibilite | `PacketFlow::try_from_timed(...)` avec la feature `parse_timing` |

`PacketFlow` contient:

```rust
pub struct PacketFlow<'a> {
    pub data_link: LinkLayer<'a>,
    pub internet: Option<Internet<'a>>,
    pub transport: Option<Transport<'a>>,
    pub application: Option<Application>,
    pub inner: Option<Box<PacketFlow<'a>>>,
}
```

Le schema de serialisation 7.0 imbrique la liaison et utilise des tags stables.
Les modeles emprunte et owned de la liaison produisent le meme JSON (les octets
du payload ne sont pas serialises):

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

## Protocoles

### Liaison

- Ethernet II
- VLAN 802.1Q
- RAW IPv4/IPv6 (`LINKTYPE_RAW`)
- Linux cooked capture v1 (`LINKTYPE_LINUX_SLL`)
- Linux cooked capture v2 (`LINKTYPE_LINUX_SLL2`)
- Adresses MAC et resolution OUI interne
- Representation IEEE 802.11 native pour les flux internes CAPWAP (pas encore
  de decodeur LINKTYPE de premier niveau)

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
| `parse_timing` | Expose `ParseTiming`, `PacketFlow::try_from_timed` et, sur la branche de developpement, `parse_timed` |

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
