 Le dépôt est un bon socle Rust : architecture modulaire, parsing majoritairement zero-copy, aucun unsafe, validations nombreuses et excellente couverture unitaire. En revanche, plusieurs
  défauts sémantiques et une chaîne de publication dangereuse doivent être corrigés avant un usage fiable sur du trafic hostile.

  Il s’agit uniquement d’une crate Rust ; aucun frontend Vue/Tauri n’est présent.

  Ethernet/VLAN → IPv4/IPv6/ARP → TCP/UDP → détection L7
                                    └→ CAPWAP → PacketFlow interne

  PCAP → benchmark_db → JSONL → ingestor → PostgreSQL/Grafana

  ## Points forts

  - Bonne séparation entre parsing, validations et erreurs : src/parse, src/checks, src/errors.
  - Parsing généralement borné avant indexation.
  - Les fragments IPv4 désactivent prudemment L4 faute de réassemblage.
  - Récursion CAPWAP limitée à quatre niveaux.
  - 18 parseurs applicatifs environ, notamment DNS, TLS, SNMP, PostgreSQL, QUIC et protocoles industriels.
  - Changelog et guide d’ajout de protocole très détaillés.
  - Aucun bloc unsafe détecté.

  ## Problèmes prioritaires

  1. Publication automatique non sécurisée. Le CI déclenche explicitement Coverage, qui déclenche Publish, alors que workflow_run effectue déjà ces enchaînements. De plus, les workflows
     réagissent à tout événement completed sans tester conclusion == success. Une PR ou un workflow échoué peut donc entraîner des tentatives multiples de publication depuis main.
     Voir .github/workflows/ci.yml:45, .github/workflows/coverage.yml:3 et .github/workflows/publish.yml:3. La publication devrait être limitée à un tag/version explicite, avec une seule
     chaîne et un garde de succès.

  2. La couche L3 ignore l’EtherType. PacketFlow transmet seulement le payload Ethernet à Internet::try_from (src/parse/mod.rs:191). Celui-ci essaie ARP, IPv4, IPv6 puis Profinet et avale
     toutes les erreurs (src/parse/internet/mod.rs:45). Un IPv4 malformé peut ainsi devenir un succès avec internet: None, tandis qu’un payload IPv4 sous un EtherType LLDP peut être mal
     classé. Il faut un try_from_parts(ethertype, payload) distinguant protocole inconnu et protocole connu mais corrompu.

  3. to_owned() perd les tunnels. PacketFlowOwned ne contient aucun champ inner et la conversion l’ignore (src/owned/mod.rs:15). to_owned() clone même toute la chaîne avant de la jeter (src/
     parse/mod.rs:157). Ajouter inner: Option<Box<PacketFlowOwned>> et convertir depuis &PacketFlow.

  4. Le chemin chronométré change le résultat. try_from_timed() force systématiquement inner: None (src/parse/mod.rs:294). Il ne se contente donc pas de ne pas mesurer les tunnels : il ne
     les retourne pas. Le parsing normal et instrumenté devraient partager une seule implémentation.

  5. IPv6 est incomplet. Les headers d’extension ne sont pas parcourus et extension_headers reçoit artificiellement &[0u8] (src/parse/internet/protocols/ipv6.rs:98). Hop-by-Hop, Routing,
     Fragment, AH et Destination Options ne peuvent donc pas conduire correctement à L4.

  6. L’ingestor peut perdre des mesures. L’offset du fichier est avancé avant la réussite de l’insertion PostgreSQL (ingestor/src/main.rs:315). En cas d’erreur SQL, il sort seulement de la
     boucle (ingestor/src/main.rs:337) : le prochain scan ignore les événements non insérés. Il faut valider l’offset après transaction réussie.

  ## Écarts fonctionnels et API

  - Le dispatcher L7 n’intègre pas HTTP ni MQTT ; DHCPv6, AMS et COTP sont commentés (src/parse/application/mod.rs:31). La liste des parseurs disponibles ne correspond donc pas à la
    détection automatique.

  - DNS ne parse que les questions ; réponses, autorités et additionnels restent toujours à None (src/parse/application/protocols/dns/mod.rs:52). La compression des noms n’est pas réellement
    implémentée.

  - Les erreurs détaillées vivent dans un module privé, alors que les parseurs publics les exposent comme types associés (src/lib.rs:106). Les consommateurs peuvent difficilement les nommer
    ou les analyser.

  - MacAddress::try_from(String) panique sur un composant hexadécimal invalide (src/parse/data_link/mac_addres/mod.rs:168). hex_stream_to_bytes panique également sur une entrée utilisateur
    (src/convert/mod.rs:60).

  - Les checksums IPv4/TCP/UDP sont lus mais non validés. Un mode configurable serait préférable à une validation obligatoire, notamment à cause de l’offload réseau.
  - La promesse « allocation-free » est trop forte : DNS, HTTP, PostgreSQL, SNMP, COTP et les tunnels allouent des Vec ou Box.
  - L’égalité et le hash ignorent les payloads. C’est cohérent pour une identité de flux, mais ce contrat devrait être explicite.
  - Le README conseille encore packet_parser = "3.0.1" alors que le manifeste est en 4.0.0 (README.md:21, Cargo.toml:1).
  - benchmark_db utilise un chemin PCAP absolu propre à une machine (benchmark_db/src/main.rs:283).
  - Aucun fuzzing ou property testing n’est configuré, alors qu’il s’agit du complément naturel aux tests unitaires pour des octets hostiles.

  ## Vérifications exécutées

  - cargo fmt --all -- --check : réussi.
  - cargo test : 590 tests + 13 doctests réussis.
  - cargo test --workspace --all-features : 598 tests + 13 doctests réussis ; les quatre binaires auxiliaires ont chacun zéro test.
  - Clippy strict sur la crate principale, toutes features : réussi.
  - Clippy strict sur tout le workspace : échec sur deux lints dans ingestor.
  - L’audit d’advisories n’a pas été rejoué localement, les outils cargo-audit/cargo-deny n’étant pas installés.

  Ordre recommandé : sécuriser la publication, corriger le routage L3, préserver inner, unifier le parsing chronométré, réparer l’ingestor, puis traiter IPv6 et la détection L7. Aucun
  fichier n’a été modifié.