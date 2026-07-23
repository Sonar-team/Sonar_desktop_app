# Plan : preuve d'exactitude des matrices par test différentiel SONAR vs TShark

> Statut : livré — rattaché au sprint de fidélité #165 (tâche « tests
> d'intégration pcap/CSV ») et prépare [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151)
> Dernière revue : 23/07/2026
> La preuve d'exactitude précède toute réintégration de la matrice dans
> l'UI (#160) : on ne réaffiche pas une matrice dont la fidélité n'est pas
> démontrée.

## Objectif

Prouver, de façon automatisée et auditable, que la matrice de flux produite
par la chaîne d'import PCAP (`convert_from_pcap_list` → `sonar_flows_core`)
est exacte : chaque flux de chaque DLT supporté est compté à l'identique
d'un oracle indépendant (TShark), flux par flux, octets et paquets.

## Principe

TShark sert d'**oracle indépendant**, mais uniquement à la génération des
fixtures : la vérité terrain est committée à côté de chaque pcap, et la CI
compare SONAR à ces fichiers **hors ligne** (cohérent avec le pilier
passif/hors-ligne et les snapshots APT — pas de TShark à installer en CI).

Trois niveaux de preuve :
1. agrégats (Σ paquets, Σ octets) ;
2. comparaison flux par flux ;
3. invariants internes (round-trip, déterminisme, tunnels).

## Constat (état au 17/07/2026)

- `script/pcap/analyze-tshark.sh` + `summarize-matrix-csv.py` existent déjà
  et vérifient les **agrégats** contre un CSV attendu. La fixture
  `src-tauri/test_files/pcaps/import/convert_pcap_to_matrice/`
  (`pcap_to_matrice.pcapng`, 15 paquets / 1008 octets, Ethernet) passe.
- Limites : outil manuel orienté documentation, rien dans `cargo test` ni
  en CI, comparaison limitée aux totaux (une matrice qui inverse deux flux
  passerait), un seul DLT couvert.
- L'arborescence `src-tauri/test_files/pcaps/{import,export}` a été créée
  le 16/07 mais la chaîne `convert_from_pcap_list` reste sans test.

## Étape 1 — Formaliser la règle de correspondance SFMS ↔ TShark

C'est le cœur du travail : les deux modèles ne sont pas alignés.

- Une ligne SFMS est **unidirectionnelle** (`mac_src → mac_dst`), les
  conversations TShark (`-z conv`) sont bidirectionnelles. La vérité
  terrain doit donc être dérivée de l'**export par paquet**
  (`tshark -T fields -e eth.src -e eth.dst -e vlan.id -e ip.src -e ip.dst
  -e tcp.srcport … -e frame.len`), agrégé par direction.
- SONAR **scinde par protocole applicatif** : dans la fixture actuelle, le
  flux 41408→443 donne 2 lignes (TCP 3 pqt/174 o + TLS 1 pqt/90 o) là où
  TShark voit 4 trames/264 octets. Le comparateur agrège les lignes SONAR
  par (mac, ip, ports, transport) avant comparaison.
- Cas particuliers à documenter : ARP (IPs tirées du payload ARP), IPs
  placeholder, MAC non-unicast, `frame.len` vs snaplen/FCS, et la colonne
  `last_seen` — exclue de l'égalité stricte, ou vérifiée contre le
  `frame.time` du dernier paquet du flux (version forte, optionnelle). La
  décision doit être explicite, pas laissée au hasard des microsecondes.

Livrable : `src-tauri/test_files/pcaps/README.md` fixant ces règles —
c'est lui qui rend la preuve auditable par un relecteur externe.

## Étape 2 — Vérité terrain machine-lisible

Étendre `analyze-tshark.sh` (ou un `gen-ground-truth.sh` sœur) pour
émettre, en plus du `.tshark.md` humain, un **`<stem>.flows.tsv`
canonique** : une ligne par flux directionnel (clé + count + bytes), trié,
avec en en-tête le SHA-256 du pcap et la version TShark.
`summarize-matrix-csv.py` fournit déjà la moitié de la logique
d'agrégation.

## Étape 3 — Corpus multi-DLT

Fixtures déjà presque toutes présentes dans `src-tauri/test_files/` :

| Fixture | Rôle |
|---|---|
| `pcaps/import/convert_pcap_to_matrice/pcap_to_matrice.pcapng` | Ethernet nominal (déjà validé en agrégat) |
| `eth_dns.pcapng`, `tlsv3.pcapng` | UDP/DNS et TLS (scission applicative) |
| `sll.pcap`, `capture_sll2.pcap` | LINUX_SLL / LINUX_SLL2 |
| `raw_ip.pcapng` | RAW/IP nu (DLT 101) |
| `ndpi_capwap.pcap` | Tunnels : invariants père/fils de `TUNNELS.md` |
| `llc.pcap`, `bluetooth_hyperx.pcapng` | **Refus explicite** de DLT non supporté — erreur typée, aucune matrice partielle |

Pour chaque pcap nominal : générer et committer `.flows.tsv` + le CSV SFMS
attendu.

## Étape 4 — Tests d'intégration Rust (pièce maîtresse)

`sonar-rust/crates/sonar-flows-core/tests/pcap_accuracy.rs` (feature
`pcap`) — dans le **source** du cœur, jamais dans `src-tauri/vendor/` ; le
desktop en bénéficie au prochain re-vendoring. Pour chaque fixture :

1. **Agrégats** : Σ`count` == nombre de trames TShark, Σ`total_bytes` ==
   data bytes.
2. **Flux par flux** : matrice canonicalisée (règle de l'étape 1) ==
   `.flows.tsv`, avec un diff lisible en cas d'échec (flux manquants / en
   trop / compteurs divergents) — c'est ce qui transforme le test en
   preuve exploitable.
3. **Round-trip** : export CSV → réimport → matrice identique ; export
   **déterministe octet-pour-octet** et égal au CSV committé (#148).
4. **Tunnels** (CAPWAP) : un `encap_id` par paire, comptabilité exacte
   père/fils, une ligne par flux.
5. **DLT non supportés** : erreur `SonarCoreError` attendue, rien
   d'intégré silencieusement.

## Étape 5 — Test côté desktop

Un test ciblé sur l'enrobage `convert_from_pcap_list`
(`src-tauri/src/commandes/import/pcap.rs`) via `tauri::test` : progression
`ImportProgress` monotone, `ImportGuard` exclusif, erreurs du cœur
fidèlement transmises au front. La preuve d'exactitude vit dans le cœur ;
ici on prouve que l'adaptateur ne dégrade rien.

Livré dans les tests de `commandes/import/pcap.rs` avec `tauri::test` : la
commande passe par le vrai handler IPC, intercepte quatre jalons d'une capture
SLL (0, 1000, 2000, fin), tente un second import pendant `Started` et vérifie
la forme frontend `import/unsupportedLinkType` sur la frontière IEEE 802.11.

## Étape 6 — Câblage CI

- Ajouter le run des tests `--features pcap` au job `sonar_rust_checks` de
  `rust-ci.yml` (vérifier que libpcap-dev y est installé — il l'est pour
  `src-tauri`).
- **Gate de fraîcheur** sans TShark : un script vérifie que le SHA-256 en
  tête de chaque `.flows.tsv`/`.tshark.md` correspond à son pcap —
  impossible de modifier une fixture sans régénérer la vérité terrain.
- Optionnel : job cron qui régénère la vérité terrain avec le vrai TShark
  pour détecter une dérive de version d'oracle (version épinglée dans les
  rapports : 4.6.6).

## Étape 7 — Ouverture vers #151

Ce harnais devient la base du corpus adversarial : pcaps tronqués,
longueurs menteuses, puis fuzzing (`cargo-fuzz` sur le lecteur pcap et le
parsing) avec les fixtures comme corpus de départ.

La cible `sonar-flows-core/fuzz/fuzz_targets/pcap_reader.rs` traverse libpcap,
le parseur et la matrice. `script/pcap/prepare-fuzz-corpus.sh` l'alimente avec
les petites fixtures multi-DLT/oracle de #168 et deux variantes tronquées
déterministes ; les crashes minimisés restent à qualifier et promouvoir dans
le corpus permanent de #151.

## Critère de réussite

Un relecteur externe peut : ouvrir le README des règles de correspondance,
régénérer la vérité terrain avec TShark, lancer
`cargo test --features pcap`, et constater que chaque flux de chaque DLT
supporté est compté à l'identique — ou lire un diff précis si ce n'est pas
le cas.
