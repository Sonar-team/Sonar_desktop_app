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

## Décisions demandées à Cyprien

1. Tranche 1 : OK pour clé de nœud `(vlan, ip)` et id stable ? (le
   graphe changera visuellement sur les relevés multi-VLAN existants)
2. Tranche 2 : contexte par relevé en préambule `#SFMS` + par flux via
   `origin` généralisé — d'accord pour le garder HORS de la clé matrice ?
3. Tranche 3 : reste dans #154 ou part dans #164 ?
4. Saisie du site/capteur : config de capture (simple) ou notion de
   « projet » du `.sonar` (plus cohérent mais couple #154 et #159) ?
