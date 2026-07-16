# Backlog SONAR — de la bêta avancée au produit Pro

> Dernière synchronisation GitHub : 15/07/2026
> Source : audit complet bêta → pro du 13/07/2026
> Règle : les issues GitHub sont la source de vérité ; ce fichier fournit la
> priorité et l'ordre d'exécution. `sprint.md` décrit uniquement le sprint actif.
> Priorisation détaillée :
> [project_management/priorisation_beta_to_pro.md](project_management/priorisation_beta_to_pro.md).

## Ordre immédiat

1. Revalider [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87)
   et rendre les skips de
   [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) visibles.
2. Corriger l'atomicité [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139).
3. Définir la comptabilité [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150).
4. Corriger le drainage [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158).
5. Qualifier avec le corpus complet #151.
6. Générer l'IPC [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142).
7. Stabiliser l'identité [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154).

#161 et #162 peuvent avancer en parallèle sur leurs sous-tâches isolées.

## Sprint actif — fidélité des données et intégrité des sessions

Suivi : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165)

- [x] **P0** [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87) — reproduire l'import infini ou le fermer avec preuve et test *(fermée le 14/07, non reproductible avec preuves ; reliquat UI dans #161)*
- [ ] **P0** [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) — supprimer les skips silencieux, puis construire le corpus complet *(skips supprimés le 14/07 — corpus nDPI + PCAPNG forgés ; reste le fuzzing)*
- [x] **P0** [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) — réserver atomiquement l'état `Importing` pendant toute conversion *(fait le 14/07)*
- [x] **P0** [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) — détecter le DLT et comptabiliser exhaustivement les résultats de parsing *(15/07 : rapport qualité visible ; identité RAW/SLL/SLL2 réimportable sans `link_details`, fusion multi-sondes préservée)*
- [x] **P0** [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) — ne perdre aucun paquet accepté à l'arrêt ou au plafond de flux *(fait le 14/07)*
- [x] **P0** [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) — générer et tester le contrat IPC Rust ↔ TypeScript *(rouverte puis refermée le 15/07 après audit : conversion `to_contract` exhaustive imposée par le compilateur, `#[ts(optional)]` sur les champs réellement omissibles, gate CI corrigée (`git status`, pas `git diff` seul), `protocol_version` opérationnel sur les trois chemins de session — voir sprint.md)*
- [ ] **P0** [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) — identité d'actif contextualisée par site, capteur, interface et VLAN
- [ ] **P1 validation** [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) — revalider immédiatement les chemins espaces/Unicode *(backend testé le 14/07 ; restent Windows et front)*
- [ ] **P1** *(sans issue — à ouvrir)* — créer les fichiers de test (pcap simples forgés + matrices CSV attendues) et les tests d'intégration couvrant import pcap, conversion pcap → matrice, export et ré-import de matrice *(arborescence `src-tauri/test_files/pcaps/` créée le 16/07 avec un premier couple pcapng/CSV ; la chaîne `convert_from_pcap_list` et `sonar-flows-core::pcap` restent sans aucun test)*

La Definition of Done détaillée est dans `sprint.md` et dans l'issue #165.

## Sprint suivant — sessions et parcours produit terminés

- [ ] **P0** [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159) — projets persistants, autosave, récupération et manifest de preuve
- [ ] **P0** [#160](https://github.com/Sonar-team/Sonar_desktop_app/issues/160) — matrice de flux de production dans le parcours principal
- [ ] **P0** [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) — intégrité frontend : doublons, imports bloqués et erreurs invisibles
- [ ] **P1** [#111](https://github.com/Sonar-team/Sonar_desktop_app/issues/111) — fiabiliser tous les parcours sauvegarde/export
- [ ] **P1** [#102](https://github.com/Sonar-team/Sonar_desktop_app/issues/102) — produire un support bundle ZIP cohérent sous Windows
- [ ] **P1** [#145](https://github.com/Sonar-team/Sonar_desktop_app/issues/145) — supprimer ou migrer les routes et vues héritées
- [ ] **P1** [#144](https://github.com/Sonar-team/Sonar_desktop_app/issues/144) — rendre les parcours principaux conformes WCAG 2.2 AA

## Sprint suivant — distribution professionnelle

- [ ] **P0** [#94](https://github.com/Sonar-team/Sonar_desktop_app/issues/94) — Authenticode, Developer ID, notarisation et Apple Silicon
- [ ] **P0** [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146) — E2E Tauri et installateurs réellement testés sur chaque OS
  - [ ] Ajouter Cypress Component + E2E navigateur pour les parcours Vue/Vite
    avec les API Tauri simulées ; conserver WebdriverIO/Tauri pour le binaire,
    l’IPC Rust et les validations natives par OS.
- [ ] **P0** [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) — quality gates et scans bloquants sur toute release taggée
- [ ] **P1** [#96](https://github.com/Sonar-team/Sonar_desktop_app/issues/96) — modèle de menace, preuve de passivité et durcissement runtime
- [ ] **P1** [#143](https://github.com/Sonar-team/Sonar_desktop_app/issues/143) — moindre privilège Tauri, chemins validés et helper de capture
- [ ] **P1** [#163](https://github.com/Sonar-team/Sonar_desktop_app/issues/163) — documentation et support d'une distribution professionnelle
- [ ] **P2 ouverte** [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138) — conserver Npcap externe, détecter/rediriger, puis consulter Nmap

## Différenciation Pro — après fermeture des P0

- [ ] **P2** [#164](https://github.com/Sonar-team/Sonar_desktop_app/issues/164) — baseline/diff, inventaire d'actifs, rapports attestables et spécification SFMS
- [ ] **P2** [#156](https://github.com/Sonar-team/Sonar_desktop_app/issues/156) — arguments de session du desktop pour orchestration et recette
- [ ] **P1** [#132](https://github.com/Sonar-team/Sonar_desktop_app/issues/132) — performance de capture sous forte charge, après fidélité
- [ ] **P3** [#133](https://github.com/Sonar-team/Sonar_desktop_app/issues/133) — publier crates.io après stabilisation des API

## Backlog GitHub historique à requalifier

Ces tickets restent ouverts et ont reçu une priorité P1 à P3. Leur périmètre
n'a pas été réécrit en détail et doit être revalidé avant intégration dans un
sprint.

### Bugs et validation

- [ ] **P1** [#97](https://github.com/Sonar-team/Sonar_desktop_app/issues/97) — validation Windows 11
- [ ] **P1** [#98](https://github.com/Sonar-team/Sonar_desktop_app/issues/98) — VAE SONAR, gate de Release Candidate
- [ ] **P2** [#107](https://github.com/Sonar-team/Sonar_desktop_app/issues/107) — paquet Debian non reproductible
- [ ] **P3** [#118](https://github.com/Sonar-team/Sonar_desktop_app/issues/118) — revalider l'ancien échec MSI, actuellement désactivé
- [ ] **P2** [#119](https://github.com/Sonar-team/Sonar_desktop_app/issues/119) — reproductibilité NSIS
- [ ] **P2** [#120](https://github.com/Sonar-team/Sonar_desktop_app/issues/120) — reproductibilité DMG
- [ ] **P2** [#121](https://github.com/Sonar-team/Sonar_desktop_app/issues/121) — robustesse des snapshots APT face aux erreurs 503

### Produit, interface et dette technique

- [ ] **P1** [#89](https://github.com/Sonar-team/Sonar_desktop_app/issues/89) — écran À propos alimenté par le build
- [ ] **P1** [#90](https://github.com/Sonar-team/Sonar_desktop_app/issues/90) — filtres cohérents matrice/graphe/export
- [ ] **P3** [#91](https://github.com/Sonar-team/Sonar_desktop_app/issues/91) — homogénéisation visuelle des sous-menus
- [ ] **P1** [#92](https://github.com/Sonar-team/Sonar_desktop_app/issues/92) — légendes évitant une mauvaise interprétation
- [ ] **P2** [#101](https://github.com/Sonar-team/Sonar_desktop_app/issues/101) — refonte visuelle des icônes
- [ ] **P1** [#109](https://github.com/Sonar-team/Sonar_desktop_app/issues/109) — typer et normaliser l'état de `ConfigPanel`
- [ ] **P2** [#112](https://github.com/Sonar-team/Sonar_desktop_app/issues/112) — retirer les logs console des chemins chauds
- [ ] **P3** [#124](https://github.com/Sonar-team/Sonar_desktop_app/issues/124) — suivis Gemini issus de la PR Node 24

## Réalisé récemment

- [x] [#135](https://github.com/Sonar-team/Sonar_desktop_app/issues/135) — gates CI Rust/frontend/core
- [x] [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136) — release atomique et contrôle de reproductibilité
- [x] [#137](https://github.com/Sonar-team/Sonar_desktop_app/issues/137) — SBOM frontend
- [x] [#140](https://github.com/Sonar-team/Sonar_desktop_app/issues/140) — pool de buffers jumbo
- [x] [#141](https://github.com/Sonar-team/Sonar_desktop_app/issues/141) — télémétrie de backpressure
- [x] [#147](https://github.com/Sonar-team/Sonar_desktop_app/issues/147) — limites mémoire live initiales
- [x] [#148](https://github.com/Sonar-team/Sonar_desktop_app/issues/148) — CSV déterministe, atomique et protégé contre l'injection de formule
- [x] [#149](https://github.com/Sonar-team/Sonar_desktop_app/issues/149) — machine d'état de capture et identifiant de session IPC
- [x] [#152](https://github.com/Sonar-team/Sonar_desktop_app/issues/152) — décision de licence AGPL
- [x] [#153](https://github.com/Sonar-team/Sonar_desktop_app/issues/153) — import et normalisation des labels
- [x] [#155](https://github.com/Sonar-team/Sonar_desktop_app/issues/155) — suppression du mode headless desktop
- [x] [#157](https://github.com/Sonar-team/Sonar_desktop_app/issues/157) — interface unifiée de gestion des labels

## Règles de sortie de bêta

- aucun P0 ouvert avant la Release Candidate ;
- aucun P1 ouvert avant la 1.0 Pro, sauf dérogation écrite et limitée ;
- chaque paquet lu est classé ou compté comme perdu avec une raison ;
- aucun travail utilisateur n'est perdu sur stop, fermeture ou crash ;
- les parcours capture/import/matrice/graphe/labels/export passent en E2E ;
- les installateurs sont signés, notarifiés et testés sur machine propre ;
- les limites, prérequis et données sensibles sont documentés.
