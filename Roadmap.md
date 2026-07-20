# Roadmap SONAR — de la bêta avancée au produit professionnel

> Dernière mise à jour : 20/07/2026 (version courante : 4.7.0, packet_parser 8.1.0)
> Sources : audits complets bêta → pro des 13/07 et 17/07/2026
> Articulation des documents : les issues GitHub sont la source de vérité ;
> `todo.md` donne la priorité et l'ordre d'exécution ; `sprint.md` décrit le
> sprint actif ; ce fichier donne la trajectoire d'ensemble.
> Version épurée pour non-développeurs et décideurs : `roadmap_simple.md`.

## Objectif général

Faire de SONAR un outil de cartographie passive de flux réseau digne d'un
usage professionnel : chaque paquet accepté est comptabilisé, aucun travail
utilisateur n'est perdu, les installateurs sont signés et testés, et les
résultats sont attestables.

---

## Phase 1 — Fidélité des données et intégrité des sessions *(sprint actif — orange)*

Suivi : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165) — détail dans `sprint.md`.

Prouver que le résultat affiché représente fidèlement toutes les données
acceptées par l'application.

- **P0 — lot livré, issue ouverte** [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) — DLT réel et rapport de parsing ; garanties transverses restantes
- **P0 — livré** [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) — état `Importing` réservé atomiquement pendant toute conversion
- **P0 — livré** [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) — aucun paquet accepté perdu à l'arrêt ou au plafond de flux
- **P0 — partiel** [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) — skips silencieux supprimés ; corpus complet et fuzzing restants
- **P0 — ouvert** [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) — identité d'actif contextualisée (site, capteur, interface, VLAN)
- **P0 — livré** [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) — contrat IPC Rust ↔ JSON ↔ TypeScript généré et testé
- **P0 — ouvert** [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166) — interblocage start/stop concurrent
- **P0 — ouvert** [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167) — imports vides et resets concurrents
- **P1 validation — ouvert** [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) — chemins Windows et frontend
- **P1 validation — ouvert** [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168) — preuve différentielle PCAP → matrice avec TShark

## Phase 2 — Sessions et parcours produit terminés

Rendre le cycle de travail complet fiable : rien ne se perd, rien ne casse
en silence.

- **P0** [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159) — projets persistants, autosave, récupération et manifest de preuve
- **P0** [#160](https://github.com/Sonar-team/Sonar_desktop_app/issues/160) — matrice de flux de production dans le parcours principal
- **P0** [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) — intégrité frontend : doublons, imports bloqués, erreurs invisibles
- **P1** [#111](https://github.com/Sonar-team/Sonar_desktop_app/issues/111) — fiabiliser tous les parcours sauvegarde/export
- **P1** [#102](https://github.com/Sonar-team/Sonar_desktop_app/issues/102) — support bundle ZIP cohérent sous Windows
- **P1** [#145](https://github.com/Sonar-team/Sonar_desktop_app/issues/145) — supprimer ou migrer les routes et vues héritées
- **P1** [#144](https://github.com/Sonar-team/Sonar_desktop_app/issues/144) — accessibilité WCAG 2.2 AA des parcours principaux

## Phase 3 — Distribution professionnelle

Livrer des binaires auxquels un client peut faire confiance sur machine
propre.

- **P0 — rouvert** [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136) — construire une fois, tester puis publier les mêmes artefacts sans remplacement
- **P0** [#94](https://github.com/Sonar-team/Sonar_desktop_app/issues/94) — Authenticode, Developer ID, notarisation, Apple Silicon
- **P0** [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146) — E2E Tauri et installateurs testés sur chaque OS supporté
- **P0** [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) — quality gates et scans bloquants sur toute release taguée
- **P1** [#96](https://github.com/Sonar-team/Sonar_desktop_app/issues/96) — modèle de menace, preuve de passivité, durcissement runtime
- **P1** [#143](https://github.com/Sonar-team/Sonar_desktop_app/issues/143) — moindre privilège Tauri, chemins validés, helper de capture
- **P1** [#163](https://github.com/Sonar-team/Sonar_desktop_app/issues/163) — documentation et support d'une distribution professionnelle
- **Suivi** [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138) — Npcap prérequis externe, veille sur la redistribution

## Phase 4 — Différenciation Pro *(après fermeture des P0)*

- [#164](https://github.com/Sonar-team/Sonar_desktop_app/issues/164) — baseline/diff, inventaire d'actifs, rapports attestables, spécification SFMS
- [#156](https://github.com/Sonar-team/Sonar_desktop_app/issues/156) — arguments de session du desktop pour orchestration et recette
- [#132](https://github.com/Sonar-team/Sonar_desktop_app/issues/132) — performance de capture sous forte charge (après la fidélité, jamais avant)
- [#133](https://github.com/Sonar-team/Sonar_desktop_app/issues/133) — publication crates.io de `sonar-flows-core` et `sonar-flows-cli` après stabilisation des API

## Backlog à requalifier

Tickets historiques ouverts, hors phases tant qu'ils ne sont pas requalifiés
(liste complète dans `todo.md`) : import PCAP (#88), validation
Windows 11 et VAE (#97, #98), reproductibilité des paquets (#107, #118,
#119, #120, #121), finitions UI (#89, #90, #91, #92, #101), dette frontend
(#109, #112, #124).

---

## Règles de sortie de bêta

- aucun P0 ouvert dans les phases 1 à 3 ;
- chaque paquet lu est classé ou compté comme perdu avec une raison ;
- aucun travail utilisateur n'est perdu sur stop, fermeture ou crash ;
- les parcours capture/import/matrice/graphe/labels/export passent en E2E ;
- les installateurs sont signés, notarifiés et testés sur machine propre ;
- les limites, prérequis et données sensibles sont documentés.

## Jalons déjà atteints

- **Lots fidélité livrés** — import infini revalidé et fermé (#87), état
  `Importing` atomique (#139), DLT/rapport qualité (#150), drainage exact
  (#158) et contrat IPC généré (#142).
- **Fiabilisation CI/release** (partielle) — gates Rust/frontend/core (#135),
  publication atomique en draft livrée mais #136 rouvert pour l'identité et
  l'immuabilité des artefacts ; SBOM frontend initial (#137).
- **Méthodologie de build sécurisé Tauri** (archivé) — méthodologie livrée,
  écarts transférés vers #94, #143, #146, #162 et #163.
- **Revue reproducible builds** (archivé) — suites actives : #107, #119, #120,
  #94 et #162.
- **Cœur domaine extrait** — `sonar-flows-core` consommé par le desktop et la
  CLI ; corrections domaine faites une seule fois dans la crate.
- **Capture et exports fiabilisés** — machine d'état de capture avec session
  IPC (#149), CSV déterministe et atomique (#148), limites mémoire live (#147),
  télémétrie de backpressure (#141), pool de buffers jumbo (#140).
- **Produit** — licence AGPL décidée (#152), import et normalisation des labels
  (#153), interface unifiée de gestion des labels (#157), suppression du mode
  headless desktop (#155, décision VISION.md).
