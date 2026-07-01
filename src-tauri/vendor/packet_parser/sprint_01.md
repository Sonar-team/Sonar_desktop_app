# Sprint 01 - Reste a faire

Ce fichier liste les ecarts restants entre la librairie actuelle et la methode de travail decrite dans `METHODE_AJOUT_PROTOCOLE.md`.

## Etat actuel

- Les erreurs non typees prioritaires ont ete remplacees par des erreurs dediees.
- Les validations inline prioritaires ont ete deplacees vers `src/checks`.
- Les nouveaux ajouts doivent maintenant suivre `TryFrom<&[u8]>`, `thiserror`, `src/errors`, `src/checks`, zero-copy et rustdoc Mermaid `packet-beta`.
- `cargo fmt` et `cargo test` passent a la date de ce sprint.

## Reste prioritaire

- Finaliser l'audit protocole par protocole pour verifier le respect complet de la methode.
- Creer une erreur dediee QUIC dans `src/errors/application/quic.rs` au lieu de passer par une erreur applicative generique.
- Deplacer ou encapsuler les dernieres validations locales encore presentes dans certains parseurs complexes, notamment GIOP, QUIC cursor, TLS record loop, OPC UA, COTP et S7Comm.
- Continuer la migration zero-copy des parseurs qui exposent encore des `Vec<u8>`, `String`, `to_vec()` ou `to_string()` dans le chemin de parsing.
- Revoir les payloads et champs variables de Bitcoin, DNS, HTTP, MQTT, QUIC, SRVLOC, COTP, TLS et S7Comm pour privilegier `&'a [u8]` ou `&'a str` valide.
- Ajouter des tests unitaires directs pour les nouveaux checks, pas seulement via les parseurs.
- Ajouter des tests de bornes pour les longueurs declarees, offsets, champs variables et payloads tronques.
- Verifier que chaque protocole expose un schema Mermaid `packet-beta` exact dans la rustdoc du type principal.
- Verifier que chaque nouveau fichier conserve l'en-tete de licence MIT du projet.

## Definition de termine

- Tous les protocoles ont un parseur `TryFrom<&[u8]>`.
- Les erreurs de protocole sont dans `src/errors/<couche>/`.
- Les validations de structure sont dans `src/checks/<couche>/`.
- Le parseur ne copie pas les payloads ou champs variables sans justification explicite.
- La rustdoc du type principal documente le format paquet avec Mermaid `packet-beta`.
- `cargo fmt` et `cargo test` passent.
