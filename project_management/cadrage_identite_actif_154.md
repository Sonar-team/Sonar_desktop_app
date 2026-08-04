# Cadrage #154 — Identité d'actif contextualisée

> Rédigé le 04/08/2026, sur main après merge de la phase A du sprint
> sessions (#159, PR #184). Cartographie du code vérifiée sur
> sonar-flows-core 0.4.0 vendorée et src-tauri courant.
> Statut : proposition à arbitrer avant implémentation.

## Constat — où est réellement le défaut

L'audit de l'issue est confirmé, mais avec une précision qui change le
plan : **les trois surfaces n'ont pas le même niveau de maladie.**

| Surface | Clé d'identité actuelle | État |
| --- | --- | --- |
| Matrice (`PacketFlowOwned`) | MAC src/dst + ethertype + **VlanTag** + IP + ports + protocoles (hash/eq sur tous les champs) | **Saine** : deux IP identiques sur deux VLAN font déjà deux flux distincts |
| Graphe (`GraphData::add_packet_flow`) | **IP stringifiée seule** (graph.rs:352-356) quand les deux IP sont valides ; `mac:<mac>` sinon | **Défaut structurel** : fusion implicite inter-VLAN/sites ; VLAN totalement absent du graphe ; `Node.id` non stable (compteur atomique) |
| Labels (`LabelStore` + `FlowMatrix.label`) | `(mac, ip)` avec précédence exact > IP seule > MAC seule (matrix.rs:630-636) | **Fragile** : la précédence « IP seule » étiquette les deux actifs d'une IP dupliquée ; `update_node_label` ne matche que la première MAC du nœud |

Autres faits structurants :

- **Site/capteur/interface n'existent nulle part dans les données.** La
  seule provenance est `origin` = noms de fichiers CSV fusionnés (vide en
  capture live et import PCAP). L'interface (`device_name`) n'est connue
  qu'au `Started` ; au moment d'intégrer un paquet, elle n'est plus
  disponible (`PacketWorker` ne porte que `session_id` + `link_type`).
- **`encap_id`** (jointure tunnel externe↔interne) est un hash des
  extrémités incluant déjà `vlan_id` : tant qu'on ne touche pas à la clé
  de la matrice, les identifiants de tunnel des exports existants restent
  valides.
- Le CSV SFMS a déjà la colonne `vlan_id` et lit par nom de colonne avec
  `#[serde(default)]` : ajouter des colonnes est rétrocompatible en
  lecture ; c'est l'écriture (nouvel en-tête) qui exige un versionnage.

## Proposition en trois tranches

### Tranche 1 — le graphe et les labels apprennent le VLAN
*(sonar-flows-core 0.5.0 ; ne touche NI la clé matrice NI encap_id NI le CSV)*

1. Clé de nœud : `(vlan_id, ip)` au lieu d'`ip` seule ; fallback L2
   `(vlan_id, mac)`. Deux IP identiques sur deux VLAN → deux nœuds.
   Le VLAN devient un champ de `Node` (contrat TS régénéré, affichage
   dans le panneau nœud et le libellé).
2. `Node.id` stable : dérivé de la clé (hash), plus du compteur global —
   prérequis pour la corrélation explicite (tranche 3) et pour des
   snapshots déterministes.
3. Labels : la précédence intègre le VLAN —
   `(vlan, mac, ip)` exact > `(mac, ip)` > `(ip)` > `(mac)` — et
   `update_node_label`/`refresh_labels` matchent sur toutes les MAC
   observées du nœud, pas la première.
4. Anomalies visibles (critère UI) : une IP présente sur plusieurs VLAN
   ou portée par plusieurs MAC est signalée (badge/panneau), pas fusionnée.
5. Tests exigés : VLAN distincts, IP dupliquées, multi-MAC, changement de
   MAC — tous réalisables dès cette tranche.

Impact migration : aucun sur SFMS ; `.sonar` reste schéma v1 (le graphe
est reconstruit depuis la matrice, qui porte déjà le VLAN). Un relevé
existant rouvert affiche simplement le graphe dé-fusionné.

### Tranche 2 — le contexte de relevé (site, capteur, interface)
*(décision de format à arbitrer AVANT d'implémenter)*

Le contexte n'étant pas dans le paquet, il faut le déclarer et le
propager. Deux niveaux :

- **Par relevé** : site/capteur/interface sont constants pendant une
  capture → portés par le préambule `#SFMS` (conforme à la décision de
  juillet : métadonnées du relevé en préambule) et par `manifest.json`
  du `.sonar` (schéma v2 — c'est la migration v1→v2 déjà prévue au
  sprint sessions). Saisie : champs projet/site/capteur dans la config,
  interface déjà connue (`device_name`).
- **Par flux, après fusion multi-sites** : dès qu'on fusionne deux
  relevés de sites différents, le contexte doit suivre chaque ligne →
  généraliser `origin` en « contexte d'origine » (site/capteur/interface
  et plus seulement nom de fichier), alimenté aussi par la capture live
  et l'import PCAP (aujourd'hui vide hors CSV). Colonne(s)
  `#[serde(default)]` → SFMS version=2 en écriture, lecture v1 intacte.

Point d'attention : si le contexte entre dans la **clé** de la matrice,
`encap_id` et le round-trip changent — ma recommandation est de le
garder en **annotation de provenance** (comme `origin`), pas dans la
clé : la clé technique (MAC+VLAN+IP+ports) suffit à séparer les flux, le
contexte sert à séparer les *actifs* au niveau graphe/inventaire.

### Tranche 3 — corrélation explicite et réversible
*(frontière avec l'epic #164 — probablement à y déplacer)*

« Même actif vu de deux sites » : fusion manuelle de deux nœuds,
tracée dans le projet (`.sonar`), réversible. Nécessite les id stables
de la tranche 1 et le contexte de la tranche 2. À cadrer avec
l'inventaire d'actifs de #164 pour ne pas construire deux modèles.

## Chaîne de livraison

Tranches 1 et 2 modifient sonar-flows-core (vendorée) : cycle complet
fix amont → publier 0.5.x sur crates.io → re-vendorer → vet, comme pour
packet_parser 9. Le desktop suit (contrat TS, UI, config). Prévoir la
même mécanique de PR que #183.

## Arbitrages de Cyprien — 04/08/2026

1. **Clé de nœud `(vlan, ip)` et id stable : validé.** Rationale : le
   VLAN est spécifique au 802.1Q, il fait partie de l'identité observée.
2. **Site/capteur HORS de la clé : validé.** Rationale décisive : le
   même paquet peut être vu par plusieurs capteurs — mettre le capteur
   dans la clé empêcherait de reconnaître ce recouvrement (dédup et
   corrélation inter-capteurs).
3. **Tranche 3 déplacée dans l'epic #164.** La corrélation explicite
   d'actifs (fusion manuelle tracée et réversible de deux nœuds) relève du
   même modèle que l'inventaire d'actifs et la baseline : la construire
   dans #154 ferait deux modèles pour la même chose. #154 se ferme sur
   les tranches 1 et 2.
4. **Saisie du site/capteur : au moment de l'arrêt (stop) ou de
   l'enregistrement, à la Wireshark** — pas de configuration a priori.
   Implication : un dialogue de qualification du relevé au stop/save
   (site, capteur ; interface déjà connue), valeurs écrites dans le
   préambule `#SFMS` et le `manifest.json` du `.sonar`, mémorisées comme
   proposition par défaut pour la fois suivante.

## Note d'implémentation

Le dépôt source de sonar-flows-core est ce même repo :
`sonar-rust/crates/sonar-flows-core` (workspace avec sonar-flows-cli et
fuzz), publié sur crates.io par `.github/workflows/publish-crates.yml`.
Cycle tranche 1 : PR sur `sonar-rust` (0.5.0) → publication → re-vendor
`src-tauri` → vet, comme pour packet_parser 9 (PR #183).

### Tranche 2, étape 1 — 04/08/2026 (sonar-flows-core 0.6.0)

Contexte de relevé au niveau du **préambule SFMS** (le niveau « par
relevé » de la tranche 2) :

- `SurveyContext { site, sensor, interface }`, tous optionnels, avec
  normalisation des saisies vides/blanches. **Clés du format en anglais**
  (`site=`, `sensor=`, `interface=`) par cohérence avec les en-têtes de
  colonnes, déjà anglais — « capteur » reste le terme d'interface.
- `SFMS_VERSION` passe à **2**. Les valeurs libres (espaces, accents,
  `=`, `%`) sont percent-encodées : le préambule reste un
  `clé=valeur` non ambigu. Un échappement corrompu est une erreur, pas
  une valeur devinée.
- Lecture v1 inchangée (contexte vide) ; un fichier v2 lu par un SONAR
  antérieur reste valide, ses clés inconnues étant déjà tolérées.
- `FlowMatrix.context` + `write_rows_to_csv_with_context` ;
  `write_rows_to_csv`/`format_preamble` conservent leur signature et
  écrivent sans contexte.
- Fixture de référence `ultimate_ethernet_sample.csv` régénérée en v2 ;
  les autres fixtures restent en v1 et couvrent la lecture ascendante.

Restent pour la tranche 2 : le dialogue de qualification au stop/save
côté desktop, le `manifest.json` v2 du `.sonar`, et la généralisation
d'`origin` en contexte par flux (fusion multi-sites).

### Tranche 1 implémentée — 04/08/2026 (sonar-flows-core 0.5.0)

- Clé de nœud `(vlan, ip)` (`l3_node_key`), repli L2 `(vlan, mac)` ;
  hors trame taguée la clé reste l'IP nue (continuité des relevés).
- Ids de nœuds **et** d'arêtes stables : hash FNV-1a de la clé
  d'identité (le même hachage que `encap_id`), plus de compteurs
  globaux — snapshots déterministes.
- `Node.vlan_id: Option<u16>` et `Node.duplicate_ip: bool` (IP portée
  par plusieurs VLAN : signalée via un index IP → nœuds, jamais
  fusionnée). Contrat TS à régénérer côté desktop au re-vendoring.
- Labels : `update_node_label` retourne désormais `Vec<GraphUpdate>`
  (une (mac, ip) sur deux VLAN étiquette les deux nœuds) et matche
  toutes les MAC observées ; `refresh_labels` essaie le résolveur sur
  chaque MAC observée. **Précision de périmètre** : le palier de
  précédence `(vlan, mac, ip)` exact demanderait le VLAN dans le store
  de labels donc dans labels.csv — contradictoire avec « ne touche pas
  au CSV » ; il part en tranche 2 avec le versionnage des formats.
- sonar-flows-cli : le départage des égalités de trafic de l'export
  Graphviz tranche sur l'identité lisible (IP/MAC), plus sur l'id.
- Ripple attendu au re-vendoring dans src-tauri : signatures
  `Node::new`/`Edge::new` (clé en premier argument), retour de
  `update_node_label` (labels/commands.rs), nouveaux champs dans le
  contrat d'événements.
