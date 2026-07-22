# Corpus PCAP ↔ TShark multi-DLT (#168)

Ce corpus provient de la crate `packet_parser` 8.1.0 maintenue dans
`Akmot9/Packet-parser`. Il couvre les quatre types de liaison actuellement
décodés par la crate vendue dans SONAR, ainsi que sa frontière de support.

| Fixture | DLT | Paquets | Provenance / transformation |
|---|---:|---:|---|
| `linux_sll.pcap` | 113 | 2 702 | copie exacte de `pcaps_exemple/sll.pcap` |
| `linux_sll2.pcap` | 276 | 779 | copie exacte de `pcaps_exemple/capture_sll2.pcap` |
| `raw_ip.pcap` | 101 | 15 | ICMP IPv4/IPv6 réels, Ethernet retiré avec `editcap -C 14 -L` |
| `vlan.pcap` | 1 | 7 | trames 2163, 2166, 2168, 2176, 2177, 2184 et 2185 de `The-Ultimate-PCAP.pcapng` |
| `industrial_ethernet.pcap` | 1 | 10 | DNS, ARP, S7COMM, NTP et OpenVPN extraits de `4SICS-GeekLounge-151020.pcap` |
| `capwap_radius.pcap` | 1 | 6 | CAPWAP-Control, DTLS et RADIUS extraits de la capture `vlan0--packet-capture…cap` |
| `capwap_management.pcap` | 1 | 2 | `capwap-association-valid.pcapng` normalisé en PCAP Ethernet |
| `capwap_data.pcap` | 1 | 2 | trames réelles ToDS/FromDS embarquées dans les golden tests de `packet_parser` |
| `unsupported_ieee80211.pcapng` | 105 | 1 | beacon réel, refus explicite attendu au niveau capture |

Les captures ICMP et 802.11 proviennent du corpus de Chris Sanders
(`<https://github.com/chrissanders/packets>`), conformément aux fichiers
`SOURCE.md` de `Packet-parser`. Les captures complètes SLL/SLL2 sont des
captures locales `tcpdump -i any` datées du 14 juillet 2026. Elles peuvent
contenir des adresses publiques et ne doivent pas être présentées comme des
captures anonymisées.

## Oracles indépendants

Chaque `*.flows.tsv` est généré par TShark 4.6.6 avec résolution de noms
désactivée. La clé directionnelle compare les adresses de liaison disponibles,
VLAN, EtherType, IP, protocole IP et ports. Les statistiques comparent le
nombre de paquets, les octets sur le fil et le dernier timestamp.

Pour CAPWAP, `capwap_data.flows.tsv` décrit le flux Ethernet/IP/UDP externe et
`capwap_data.inner.flows.tsv` décrit le niveau 802.11/LLC/IP/TCP interne. Les
tests imposent en plus un même `encap_id` hexadécimal aux deux niveaux et aux
deux sens. La capture d'association contient des trames de gestion 802.11 :
elle doit rester un flux CAPWAP externe sans inventer de conversation interne.

Chaque oracle contient la version TShark, le SHA-256 de sa capture, son domaine
de projection et ses totaux. Le test Rust recalcule le SHA-256 hors ligne avant
toute comparaison afin qu'une capture modifiée rende immédiatement l'oracle
périmé.

Le corpus et tous ses oracles se reconstruisent depuis un clone voisin de la
crate avec :

```shell
script/pcap/build-tshark-corpus.sh ../Packet-parser
```

La règle reste stricte : un nouveau DLT doit recevoir une projection explicite
dans `generate-common-flows.py`; il n'est jamais assimilé silencieusement à
Ethernet.
