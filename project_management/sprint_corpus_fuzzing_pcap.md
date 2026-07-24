# Sprint : qualification du corpus PCAP/PCAPNG et fuzzing borné

> Statut : à démarrer — scoping réalisé le 25/07/2026
> Dernière revue : 25/07/2026
> Rattaché au sprint de fidélité #165 ; poursuit le harnais livré par
> `plan_exactitude_matrices_tshark.md` (#168), qui ouvrait déjà explicitement
> vers ce périmètre.
>
> Suivi GitHub: [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151)
> — dépend du contrat de résultat de [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150)

## Objectif

Fermer les six critères d'acceptation encore ouverts de #151 : corpus
qualifié exhaustif (y compris loopback), malformations couvertes
systématiquement, comptabilité par catégorie de rejet, fuzzing borné câblé
en CI, remontée des seeds de régression, et audit de sensibilité du corpus.

## Constat (scoping du 25/07/2026)

Le trou n'est pas « tout construire depuis zéro » : une bonne partie de
l'infrastructure existe déjà mais n'est pas terminée ou pas branchée.

| Critère #151 | État réel |
|---|---|
| Corpus qualifié (Ethernet, VLAN, tunnels, SLL/SLL2, RAW, loopback, PCAPNG multi-interface) | Tout couvert **sauf loopback (DLT_NULL/LOOP)** — aucune fixture trouvée |
| Fichiers tronqués/longueurs incohérentes/trames malformées exhaustifs | Partiel : troncature testée sur **une seule source** (`vlan.pcap`), pas systématique par DLT ni par type de malformation |
| Résultats attendus par catégorie (lu/décodé/rejeté) | La plomberie existe (`PcapFileReport{packets, parse_ok, parse_errors}`), mais 3 compteurs plats, pas de détail par cause de rejet |
| Fuzz/property tests bornés en CI | Le fuzz target existe déjà (`sonar-rust/crates/sonar-flows-core/fuzz/fuzz_targets/pcap_reader.rs`, ajouté en `e1833bbc`) mais **n'est câblé dans aucun workflow CI** — exécution manuelle uniquement, via `script/pcap/prepare-fuzz-corpus.sh` |
| Seeds de régression du fuzzing → corpus permanent | Aucun mécanisme automatique de remontée |
| Corpus audité sans données sensibles | Pas encore fait |

Détail par fichier :

- Corpus : `src-tauri/test_files/pcaps/import/pcap_tshark_corpus/` (oracle
  #168) + `ultimate_ethernet_sample/` + quelques fixtures racine.
  DLTs couverts : Ethernet, VLAN, Linux SLL/SLL2, RAW, tunnels CAPWAP
  (`encap_id`), rejet DLT non supporté (IEEE 802.11). Le multi-interface
  PCAPNG est **forgé en code** (`pcapng_shb`/`pcapng_idb`/`pcapng_epb` dans
  `sonar-rust/crates/sonar-flows-core/src/pcap.rs`), pas stocké en fixture.
- Fuzz target : `libfuzzer-sys`, écrit des octets arbitraires en `.pcap`
  temporaire, appelle `sonar_flows_core::pcap::append_pcap_file` (libpcap →
  validation DLT → `packet_parser` → `FlowMatrix`) ; ne s'intéresse qu'aux
  panics/UB, pas aux erreurs de parsing (normales). Cible le lecteur/parseur
  du cœur, pas le chemin IPC Tauri.
- Malformations déjà testées : `truncated_pcap_is_an_error_not_an_eof`,
  `zero_length_packets_terminate_with_explicit_accounting`,
  `mixed_dlt_pcapng_fails_explicitly`, `mixed_snaplen_pcapng_fails_explicitly`
  (`sonar-flows-core/src/pcap.rs`) ; `pcap_fails_on_truncated_file_without_writing_output`
  (`sonar-flows-cli/tests/cli.rs`) ; `truncated_pcap_returns_read_error`
  (`src-tauri/src/commandes/import/pcap.rs`).
- CI : `.github/workflows/rust-ci.yml` couvre fmt/clippy/tests/audit/vet/deny/
  udeps et l'oracle `pcap_accuracy`, mais aucun job n'exécute `cargo fuzz`.

## Périmètre du sprint

1. Câbler le fuzz target existant dans un job CI borné en temps.
2. Ajouter la fixture loopback manquante (DLT_NULL) + son oracle TShark.
3. Étendre la couverture troncature/malformation à plusieurs DLTs et
   plusieurs types de corruption, pas seulement `vlan.pcap`.
4. Détailler `PcapFileReport` avec une comptabilité par catégorie de rejet,
   sans casser le contrat de résultat de #150.
5. Auditer le corpus commité pour l'absence de données réseau sensibles.
6. Formaliser la remontée des seeds de régression fuzzing vers le corpus
   permanent versionné.

## Fichiers candidats

- `sonar-rust/crates/sonar-flows-core/fuzz/fuzz_targets/pcap_reader.rs`
- `script/pcap/prepare-fuzz-corpus.sh`
- `sonar-rust/crates/sonar-flows-core/src/pcap.rs`
- `sonar-rust/crates/sonar-flows-core/tests/pcap_accuracy.rs`
- `src-tauri/test_files/pcaps/import/pcap_tshark_corpus/`
- `.github/workflows/rust-ci.yml`

## Risques

- Un budget de fuzzing CI trop long ralentit chaque push ; trop court ne
  trouve rien — calibrer sur un temps fixe (ex. 120 s) plutôt qu'un nombre
  d'itérations, pour rester prévisible en durée de CI.
- Détailler `PcapFileReport` par catégorie de rejet peut toucher le contrat
  IPC généré de #150 (#142) — vérifier avant d'étendre le type public.
- Un crash trouvé par le fuzzer et minimisé doit être qualifié (vrai bug vs
  faux positif de l'outillage) avant d'entrer dans le corpus permanent —
  sinon le corpus se remplit de bruit.
- Le forgeage en code du multi-interface PCAPNG (plutôt qu'une fixture
  stockée) est un choix déjà fait et documenté ; ne pas le dupliquer en
  fixture sans raison.

## Critères d'acceptation

Repris tels quels des six cases non cochées de #151 :

- [ ] Le corpus qualifié couvre Ethernet, VLAN, tunnels, SLL/SLL2, RAW,
      loopback et PCAPNG multi-interface.
- [ ] Fichiers tronqués, longueurs incohérentes et trames malformées sont
      couverts exhaustivement.
- [ ] Les résultats attendus incluent paquets lus, décodés et rejetés par
      catégorie.
- [ ] Des fuzz/property tests du parseur et de l'import tournent avec un
      budget CI borné.
- [ ] Les seeds de régression issus du fuzzing rejoignent le corpus.
- [ ] Le corpus complet est audité comme dépourvu de données réseau
      sensibles.

## Tâches de sprint

### SP-01 - Câbler le fuzz target en CI
- Ajouter un job (ou une étape du job `sonar_rust_checks`) qui installe
  `cargo-fuzz`, régénère le corpus de seeds via
  `script/pcap/prepare-fuzz-corpus.sh`, et lance
  `cargo fuzz run pcap_reader -- -max_total_time=120`.
- Le job échoue explicitement sur crash/UB détecté, avec l'artefact minimisé
  attaché au run.

### SP-02 - Fixture loopback (DLT_NULL/LOOP)
- Générer ou récupérer une capture loopback réelle, la documenter au même
  niveau que les fixtures SLL/SLL2 existantes.
- Générer l'oracle `.flows.tsv` associé (`script/pcap/analyze-tshark.sh`)
  et l'intégrer à `pcap_accuracy.rs`.

### SP-03 - Étendre la couverture des malformations
- Dupliquer les scénarios de troncature (header global, dernier
  enregistrement, longueur incohérente) sur au moins un fichier par famille
  de DLT déjà couverte (Ethernet, SLL/SLL2, RAW, tunnel).
- Documenter la matrice DLT × type de malformation dans
  `src-tauri/test_files/pcaps/README.md`.

### SP-04 - Comptabilité par catégorie de rejet
- Étudier si `PcapFileReport` peut être étendu (cause de rejet : DLT non
  supporté, trame tronquée, longueur incohérente, erreur de parsing L3/L4…)
  sans casser le contrat IPC de #150.
- Si extension possible : propager la catégorie jusqu'au rapport final
  visible (#158).

### SP-05 - Remontée des seeds de régression
- Définir où les crashes minimisés du fuzzer atterrissent dans le dépôt
  (probablement un sous-dossier versionné du corpus, distinct des seeds de
  départ).
- Documenter la procédure : qualifier le crash, le réduire, le committer,
  vérifier qu'il est rejoué par `pcap_accuracy`/le fuzz target en régression.

### SP-06 - Audit de sensibilité du corpus
- Relire chaque fixture commitée (IPs réelles, hostnames, contenu
  applicatif) et confirmer qu'il s'agit uniquement de trafic public/synthé-
  tique (corpus nDPI, générateurs déterministes).
- Documenter le résultat de l'audit dans
  `src-tauri/test_files/pcaps/README.md`.
