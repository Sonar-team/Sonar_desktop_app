# Changelog

Tous les changements notables du projet seront documentes dans ce fichier.

Le format suit l'esprit de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), avec des sections simples par type de changement.

## [Unreleased]

## [6.0.0] - 2026-07-12

Durcissement de la detection MQTT : sur le corpus de pcaps du depot, un
tiers des etiquettes MQTT etaient des faux positifs (payloads binaires
quelconques dont le premier octet mimait un fixed header).

### Rupture

- `MqttPacket::try_from` valide strictement MQTT 3.1/3.1.1 (avec tolerance
  v5) : nom de protocole `MQTT`/`MQIsdp` et niveau coherent exiges dans le
  CONNECT (bit reserve des connect flags controle), remaining length par
  type (acks == 2 ou forme v5 valide, PING/DISCONNECT vides ou reason code
  v5), packet id non nul, codes CONNACK/SUBACK/UNSUBACK verifies, QoS 3
  rejete, topic de PUBLISH UTF-8 non vide sans wildcard ni caractere de
  controle, payload de SUBSCRIBE/UNSUBSCRIBE decoupe en entrees exactes.
- `MqttPacket::try_from` exige que le buffer entier soit une suite de
  paquets MQTT valides : les segments TCP coalesces (plusieurs messages)
  restent acceptes, les octets residuels non-MQTT sont rejetes.
- PUBLISH QoS > 0 : le packet id fait desormais partie du
  `variable_header` (conformite spec) et non plus du `payload`.
- CONNECT v3.1 `MQIsdp` : variable header de 12 octets (il etait fige a
  10, faux pour ce nom de protocole).
- `MqttError` gagne neuf variantes (`InvalidProtocolName`,
  `InvalidProtocolLevel`, `InvalidReservedConnectFlag`, `InvalidQos`,
  `ZeroPacketId`, `InvalidRemainingLength`, `InvalidReasonCode`,
  `InvalidTopic`, `MalformedSubscriptionPayload`) : les `match` exhaustifs
  doivent etre completes.
- `checks::application::mqtt::variable_header_len` prend le premier octet
  du fixed header en plus : `(packet_type, first_byte, body)`.

### Corrige

- Detection MQTT : 0 faux positif sur le corpus de pcaps (20 avant), les
  38 trames MQTT reelles restent detectees, aucun effet sur les autres
  protocoles.

### Ajoute

- Exemple `scan_pcaps` : compte les protocoles applicatifs detectes par
  fichier pcap/pcapng (`cargo run --example scan_pcaps -- <dossier>
  [--focus PROTO]`), utile pour auditer les faux positifs.
- Golden tests sur trames reelles : session MQTT v3.1 complete (CONNECT,
  CONNACK, SUBSCRIBE, PUBLISH, PINGREQ) et quatre trames sosies qui
  produisaient les faux positifs.
- Captures de reference : `pcaps_exemple/protocols/mqtt/` (session Paho
  vers m2m.eclipse.org:1883) et dossiers arp/dhcp/dns/icmp/ip/tcp/
  ieee80211 issus du depot de Chris Sanders (citation dans les
  `SOURCE.md`).
- Le paquet crates.io exclut desormais `pcaps_exemple/` et
  `integration_test/` (2,8 Mo de captures hors tarball).

## [5.0.0] - 2026-07-11

Corrections issues de l'audit `analyse.md` : semantique de parsing, chaine de
publication et completude protocolaire.

### Rupture

- La couche L3 est desormais routee par l'EtherType via
  `Internet::try_from_parts(ethertype, payload)` : un EtherType inconnu donne
  `internet: None` comme avant, mais un paquet **corrompu** sous un EtherType
  connu (IPv4/IPv6/ARP/Profinet) n'est plus avale silencieusement.
  `Internet::try_from(&[u8])` (probing sans EtherType) reste disponible.
- Degradation gracieuse sur couche corrompue : `PacketFlow` gagne un champ
  `corrupted: Option<CorruptedLayer>` (`layer: Internet | Transport`,
  `error: String`). Quand une couche reconnue (par l'EtherType ou le champ
  protocole IP) contient des octets invalides, les couches au-dessus restent
  remplies, la couche fautive et celles du dessous valent `None`, et la
  corruption est rapportee — `try_from` n'echoue plus que si le L2 lui-meme
  est illisible. Meme champ sur `PacketFlowOwned`. (Auparavant, un L4
  corrompu faisait echouer tout le parsing.)
- `Ipv6Packet` : les extension headers (Hop-by-Hop, Routing, Fragment, AH,
  Destination Options) sont maintenant parcourus. `extension_headers` contient
  les vrais octets de la chaine (plus jamais `&[0]`), `payload` pointe apres
  les extensions, et les nouveaux champs `transport_protocol` /
  `is_fragmented()` exposent le protocole L4 final (aucun L4 n'est parse pour
  les fragments, comme en IPv4).
- `PacketFlowOwned` gagne un champ `inner: Option<Box<PacketFlowOwned>>` :
  `to_owned()` preserve desormais les tunnels (CAPWAP…) au lieu de les perdre,
  et la conversion se fait depuis `&PacketFlow` sans clone intermediaire.
- `MacAddress::try_from(String)` retourne `MacParseError::InvalidComponent`
  sur un composant hexadecimal invalide au lieu de paniquer.
- HTTP : la methode doit etre une methode standard (RFC 9110) et la version
  doit commencer par `HTTP/` (`InvalidMethod`, `InvalidVersion`), sinon des
  payloads texte quelconques etaient classes HTTP.
- DNS : les messages annoncant des records absents ou utilisant des pointeurs
  de compression invalides (boucle, reference avant) sont maintenant rejetes
  (`InvalidCompressionPointer`, `ReservedLabelType`).

### Ajoute

- DNS complet : les sections answers, authorities et additionals sont parsees
  (elles restaient toujours `None`), avec support de la compression de noms
  RFC 1035 §4.1.4 bornee contre les boucles de pointeurs hostiles.
- Detection L7 : HTTP et MQTT rejoignent le dispatcher applicatif ; DHCPv6,
  COTP (ISO-TSAP 102) et AMS/ADS (48898/48899) sont detectes sur leurs ports
  standards (signatures trop faibles pour du probing a l'aveugle).
- `checks::checksum` : validation **opt-in** des checksums IPv4, TCP et UDP
  (`verify_ipv4_header_checksum`, `verify_tcp_checksum`,
  `verify_udp_checksum`) — le parsing ne valide toujours rien par defaut a
  cause de l'offload materiel.
- `convert::try_hex_stream_to_bytes` : variante sans panic de
  `hex_stream_to_bytes` (`HexStreamError`).
- Le module `errors` est public : les erreurs par couche
  (`errors::internet::InternetError`, etc.) sont nommables par les
  consommateurs ; `InternetError` gagne les variantes `Ipv4Error`, `Ipv6Error`
  et `ProfinetError`.
- Fuzzing : cible cargo-fuzz `fuzz/` avec trois harnais (`parse_packetflow`,
  `parse_dns`, `parse_application`).

### Corrige

- `PacketFlow::try_from_timed` retourne exactement le meme resultat que le
  parsing normal, tunnels inclus (il forcait `inner: None`) ; les deux chemins
  partagent les memes fonctions de couche.
- Publication crates.io : plus de double declenchement CI→Coverage, garde
  `conclusion == success` sur Coverage, et publication uniquement sur tag
  `vX.Y.Z` (avec verification tag/version) ou dispatch manuel.
- Ingestor : l'offset JSONL n'est valide qu'apres insertion PostgreSQL
  reussie ; plus aucune mesure perdue quand PostgreSQL est indisponible.
- `benchmark_db` : le repertoire des PCAP se passe en argument CLI ou via
  `PCAP_DIR` (plus de chemin absolu code en dur).
- README/README-fr : version d'installation corrigee (`4.0.0`).
- Documentation : la promesse « allocation-free » est precisee (zero-copy
  L2/L3/L4 ; DNS, HTTP, SNMP et les tunnels allouent), et le contrat
  d'egalite/hash (identite de flux, payloads ignores) est documente.

## [4.0.0] - 2026-07-09

Sprint 01 : mise en conformite complete avec `METHODE_AJOUT_PROTOCOLE.md`
(zero-copy, erreurs typees, validations dans `src/checks`).

### Rupture

- Migration zero-copy des parseurs qui copiaient encore des octets du paquet :
  les structures suivantes gagnent un lifetime et empruntent leurs champs au
  paquet source (`&'a [u8]` / `&'a str`) au lieu de posseder `Vec<u8>`/`String` :
  `HttpRequest<'a>`, `GiopPacket<'a>` (et `GiopRequest`, `ServiceContext`,
  `TargetAddress`), `QuicPacket<'a>` (et `ConnectionId`, `CryptoFrame`,
  `QuicFrame`, `QuicPayload`), `SrvlocPacket<'a>`, `BitcoinPacket<'a>`,
  `MqttPacket<'a>`, `CotpHeader<'a>` (et `CotpParameter`), `DhcpPacket<'a>`.
- `NtpPacket` : `Refid::KissCode` et `Refid::ClockSource` passent de `String`
  a `[u8; 4]` (avec accesseur `Refid::code()` et `Display`).
- `BitcoinError` et `MqttError` migrent vers `thiserror` (messages identiques).
- GIOP : variante stringly `GiopError::Other` supprimee au profit de variantes
  typees (`UnknownTargetDiscriminator`, `InvalidServiceContextCount`).
- Noms de protocoles corriges dans la detection applicative : `"SRVLOCK"`
  devient `"SRVLOC"` et `"QUIQ"` devient `"QUIC"` (tout consommateur qui
  matche sur ces chaines doit se mettre a jour).
- SRVLOC : la longueur declaree dans l'en-tete doit desormais correspondre
  exactement a la taille du payload, et le code fonction doit exister pour la
  version (RFC 2165 / RFC 2608).
- DHCP : une zone options non vide doit commencer par le magic cookie
  RFC 2131 (`0x63825363`) ; les options vides (BOOTP pur) restent acceptees.

### Ajoute

- Erreur dediee `QuicError` dans `src/errors/application/quic.rs` (remplace
  l'erreur applicative generique dans le parseur QUIC).
- Validations extraites des parseurs vers `src/checks` pour GIOP, QUIC
  (curseur borne + varint RFC 9000), TLS (boucle de records), OPC UA (taille
  de chunk), COTP (regles TPDU/TSAP) et S7Comm (bornes parametres/items).
- Environ 130 tests supplementaires : tests unitaires directs des checks,
  tests de bornes (longueurs declarees, offsets, payloads tronques) et tests
  de provenance de pointeur garantissant le zero-copy (598 tests au total).
- En-tete de licence MIT ajoute aux 14 fichiers qui ne l'avaient pas.

### Corrige

- Issue #3 : les paquets DHCP ne sont plus classifies `SRVLOCK`. DHCP est
  ajoute a la chaine de detection applicative (avant SRVLOC), et le parseur
  SRVLOC durci rejette les payloads BOOTP dont l'op code mimait une version
  SLP. Non-regression verifiee avec les trames reelles de `dhcp.pcap`.
- GIOP : un compteur de service contexts forge ne peut plus declencher une
  allocation demesuree (validation avant `Vec::with_capacity`).
- MQTT : un varint de longueur restante tronque renvoie desormais
  `MalformedRemainingLength` au lieu de `RemainingLengthOverflow`.
- NTP : suppression des `unwrap()` du chemin de parsing et gardes de longueur
  sur root delay / root dispersion.
- La reconstruction des noms DNS (labels + compression RFC 1035) reste la
  seule allocation du chemin de parsing, documentee comme justifiee par le
  protocole.

## [3.0.0] - 2026-07-07

### Ajoute

- Decapsulation recursive des tunnels : un paquet encapsule produit desormais
  plusieurs niveaux de flux via le nouveau champ `PacketFlow.inner`
  (`Option<Box<PacketFlow>>`) et la methode `PacketFlow::flatten()`.
- Support **CAPWAP-Data** (UDP 5247) transportant **IEEE 802.11** (ToDS/FromDS,
  WDS, QoS, gestion du byte-swap Frame Control des captures Cisco) puis
  **LLC/SNAP** jusqu'a la couche L3 interne.
- Le tunnel detecte est reporte comme protocole applicatif de la ligne externe
  (ex. `"CAPWAP"`).
- Garde-fou de profondeur (`MAX_TUNNEL_DEPTH = 4`) et degradation gracieuse :
  un contenu interne illisible (DTLS chiffre, tronque, non-SNAP) ne fait jamais
  echouer le parsing, seule la ligne externe est produite.
- Deux tests de reference sur trames reelles (CAPWAP ToDS/en-tete 16 octets et
  FromDS/en-tete 8 octets).

### Rupture

- `PacketFlow` gagne un champ public `inner` : toute construction par litteral
  de `PacketFlow` doit desormais renseigner ce champ (`inner: None` par defaut).

### Validation

- `cargo test` passe (435 tests unitaires + 13 doctests), `cargo fmt` et
  `cargo clippy` propres.

## [1.5.5] - 2026-06-29

### Change

- Detection PostgreSQL basee sur la structure du payload TCP au lieu du port 5432.
- Alignement du parseur PostgreSQL sur les formats de messages frontend/backend pour les StartupMessage 3.0/3.2 et les CancelRequest a cle secrete variable.

### Corrige

- Reduction des faux positifs PostgreSQL en exigeant une signature applicative forte avant de labelliser le payload.

### Validation

- `cargo test` passe avec 297 tests unitaires et 13 doctests.

## [1.5.4] - 2026-06-26

### Ajoute

- Ajout du parsing PostgreSQL, detecte sur le port standard et expose via les couches application, erreurs et validations dediees.

### Change

- Le pipeline benchmark/ingestion/dashboard identifie maintenant le code de la crate avec une empreinte BLAKE3 de `src/` (`crate_code`) au lieu d'un numero de version hardcode.
- Les fichiers JSONL de benchmark incluent `crate_code` et sont nommes par PCAP, code de crate et `run_id`, pour que l'ingestor voie immediatement chaque nouveau run.

### Validation

- `cargo test` passe avec 283 tests unitaires et 13 doctests.

## [1.5.3] - 2026-06-25

### Corrige

- `ParseTiming.total_ns` est maintenant renseigne pour toute tentative de parsing, y compris quand le parsing echoue en L2, L3 ou L4.
- Les paquets IPv4 fragmentes ne declenchent plus de tentative de parsing L4; la couche internet est conservee avec `payload_protocol: None`.
- Le parsing L4 sur fragment IPv4 necessite un reassemblage IP, non fourni par la crate.

### Validation

- `cargo test` passe avec 278 tests unitaires et 13 doctests.
- `cargo test --features parse_timing` passe avec 286 tests unitaires et 13 doctests.

## [1.5.2] - 2026-06-25

### Corrige

- `ParseTiming.total_ns` mesure maintenant toute tentative de parsing, y compris quand le parsing echoue en L2, L3 ou L4.
- Les fragments IPv4 non initiaux ne sont plus interpretes comme des paquets de transport complets; la couche internet est conservee avec `payload_protocol: None`.

### Validation

- `cargo test` passe avec 274 tests unitaires et 13 doctests.
- `cargo test --features parse_timing` passe avec 282 tests unitaires et 13 doctests.

## [1.5.1] - 2026-06-24

### Corrige

- Renommage des champs `protocol` aplatis dans `PacketFlowOwned` pour eviter les collisions JSON entre les couches internet, transport et application.

## [1.5.0] - 2026-06-23

### Ajoute

- Ajout du parsing EtherNet/IP encapsulation, detecte sans dependance au port.

### Validation

- `cargo fmt --check` passe.
- `cargo test` passe avec 271 tests unitaires et 13 doctests.

## [1.4.0] - 2026-06-23

### Ajoute

- Ajout de `METHODE_AJOUT_PROTOCOLE.md`, qui documente la methode de travail pour ajouter un nouveau protocole.
- Ajout d'une strategie zero-copy pour les nouveaux parseurs : payloads et champs variables en references, pas de copies inutiles dans le chemin de parsing.
- Ajout d'une exigence de rustdoc avec schema Mermaid `packet-beta` pour le type principal de chaque nouveau protocole.
- Ajout d'erreurs applicatives dediees pour HTTP, DHCP, COTP et S7Comm.
- Ajout de modules `src/checks/application/*` et `src/checks/internet/profinet.rs` pour centraliser les validations des parseurs.
- Ajout de modules d'erreurs dedies pour AMS, GIOP, Modbus/TCP, OPC UA, SRVLOC, TLS et Profinet.
- Ajout de checks dedies pour ARP, IPv4, IPv6, UDP, DHCPv6, QUIC, DNS, Bitcoin, MQTT, Modbus/TCP et SRVLOC.
- Ajout du parsing SNMP v1/v2c/v3 avec detection UDP 161/162, PDU standards et varbinds.

### Change

- Alignement des parseurs HTTP, DHCP, COTP et S7Comm vers une interface `TryFrom<&[u8]>`.
- Remplacement des erreurs de parsing non typees (`bool`, `&'static str`) par des erreurs dediees.
- Conservation des fonctions helper existantes quand elles restent utiles, mais avec des types d'erreur explicites.
- Correction d'un commentaire Bitcoin obsolete qui mentionnait encore un retour `bool`.
- Deplacement des validations nommees `check_*`, `validate_*` et `ensure_*` hors de `src/parse`.
- Deplacement des enums d'erreur restantes hors des fichiers de parsing vers `src/errors`.
- Mise a jour des parseurs AMS, Bitcoin, COTP, DHCP, DNS, GIOP, HTTP, Modbus/TCP, MQTT, OPC UA, S7Comm, SRVLOC, TLS et Profinet pour utiliser les modules `checks` et `errors`.
- Migration des validations inline prioritaires hors des parseurs Data Link, Internet, Transport, DNS, DHCPv6, QUIC, Bitcoin, MQTT, Modbus/TCP et SRVLOC.
- Deplacement de la validation `dns_flags` depuis le module de parsing DNS vers `src/checks/application/dns.rs`.

### Validation

- `cargo fmt` passe.
- `cargo test` passe avec 264 tests unitaires et 13 doctests.

## Historique avant changelog

Les versions precedentes n'etaient pas encore documentees dans un changelog dedie. Consulter l'historique Git pour les changements plus anciens.
