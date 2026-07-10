# Changelog

Tous les changements notables du projet seront documentes dans ce fichier.

Le format suit l'esprit de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), avec des sections simples par type de changement.

## [Unreleased]

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
