# Sprint 01 - Termine

Ce fichier listait les ecarts entre la librairie et la methode de travail decrite dans `METHODE_AJOUT_PROTOCOLE.md`. Le sprint est solde avec la version 4.0.0 (voir `CHANGELOG.md`).

> Suivi GitHub : [issue #16](https://github.com/Akmot9/Packet-parser/issues/16).

## Fait

- Erreurs non typees remplacees par des erreurs dediees (`thiserror`), y compris QUIC (`src/errors/application/quic.rs`), Bitcoin et MQTT.
- Validations inline deplacees vers `src/checks` pour GIOP, QUIC (curseur + varint), TLS (boucle de records), OPC UA, COTP et S7Comm.
- Migration zero-copy terminee : HTTP, GIOP, QUIC, SRVLOC, Bitcoin, MQTT, COTP, DHCP et NTP empruntent leurs champs au paquet (`&'a [u8]` / `&'a str`). Les noms de protocoles sont des `&'static str` (commit 9c2aa35).
- Seule allocation restante dans le chemin de parsing : la reconstruction des noms DNS (labels + compression RFC 1035), documentee comme justifiee par le protocole dans `src/parse/application/protocols/dns/dns_queries/mod.rs`.
- Tests unitaires directs des checks et tests de bornes ajoutes (~130 tests, 598 au total), dont des tests de provenance de pointeur garantissant le zero-copy.
- Chaque protocole expose un schema Mermaid `packet-beta` dans la rustdoc de son type principal (verifie).
- En-tete de licence MIT present sur tous les fichiers de `src/` (14 fichiers corriges).
- `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings` et `cargo test` passent.

## Definition de termine (atteinte)

- Tous les protocoles ont un parseur `TryFrom<&[u8]>`.
- Les erreurs de protocole sont dans `src/errors/<couche>/`.
- Les validations de structure sont dans `src/checks/<couche>/`.
- Le parseur ne copie pas les payloads ou champs variables sans justification explicite.
- La rustdoc du type principal documente le format paquet avec Mermaid `packet-beta`.
- `cargo fmt` et `cargo test` passent.
