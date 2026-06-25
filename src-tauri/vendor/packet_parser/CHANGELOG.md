# Changelog

Tous les changements notables du projet seront documentes dans ce fichier.

Le format suit l'esprit de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), avec des sections simples par type de changement.

## [Unreleased]

### Change

- Le pipeline benchmark/ingestion/dashboard identifie maintenant le code de la crate avec une empreinte BLAKE3 de `src/` (`crate_code`) au lieu d'un numero de version hardcode.
- Les fichiers JSONL de benchmark incluent `crate_code` et sont nommes par PCAP, code de crate et `run_id`, pour que l'ingestor voie immediatement chaque nouveau run.

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
