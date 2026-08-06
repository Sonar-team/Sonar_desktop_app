# Backlog SONAR — de la bêta avancée au produit Pro

> Dernière synchronisation GitHub : 06/08/2026 soir (v4.11.0, post-triage)
> Source : audit complet bêta → pro du 13/07/2026
> Règle : les issues GitHub sont la source de vérité ; ce fichier fournit la
> priorité et l'ordre d'exécution. `sprint.md` décrit uniquement le sprint actif.
> Priorisation détaillée :
> [project_management/priorisation_beta_to_pro.md](project_management/priorisation_beta_to_pro.md).

## Baromètre « app pro » — 06/08/2026

Dérivé des règles de sortie de bêta (bas de ce fichier) : **6 P0 et
8 P1 ouverts** après le triage du 06/08 (fermées sur preuves : #88, #150,
#154, #159, #165 ; « not planned » : #91, #101, #156 — reliquats transférés
vers #111, #146 et #164, aucune issue ouverte). La Release Candidate exige
0 P0 ; la 1.0 Pro exige 0 P1. Avancements estimés par étape de
`roadmap_simple.md` :

| Étape | Avancement | Restant (issues) |
| --- | --- | --- |
| 1. Résultats fiables | ~90 % | #154 (tranche 2 : câblage desktop), #151 (corpus/fuzzing), #150 (reliquat), #88 |
| 2. Ne jamais perdre son travail | ~60 % | #159 (phases B/C), #160, #111, #102 (reliquat), #144 |
| 3. Installation professionnelle | ~25 % | #94, #146, #136, #162, #163, #96, #143, #98 |
| 4. Analyse différenciante | 0 % | #164 (+ tranche 3 de #154), #156, #132 (après les P0) |

Acquis depuis la dernière sync (15/07 → 04/08) : intégrité frontend
(#161) et erreurs enfin toujours visibles (audit du 01/08 corrigé le
02/08, `b3d42a07`), routes mortes supprimées (#145), décision Npcap
actée (#138 : prérequis externe, fermée le 20/07), releases historiques
assainies (#169), interblocages démarrage/arrêt (#166) et imports
vides/resets concurrents (#167) corrigés, oracle TShark livré (#168),
Windows 11 validé (#97), packet_parser 9.0.0 intégré via
sonar-flows-core 0.4.0 (PR #183), sessions persistantes phase A
(#159 : format `.sonar`, dirty state, autosave, récupération crash,
projets récents — PR #184), Depends libpcap du .deb corrigé (#175,
PR #185, fermée le 04/08), identité de nœud (vlan, ip) + ids stables
livrée (tranche 1 de #154, sonar-flows-core 0.5.0, PR #186/#187 ;
tranche 3 déplacée vers #164 — cadrage dans
`project_management/cadrage_identite_actif_154.md`).

Acquis du 05-06/08 : scénario E2E X11 resynchronisé avec l'app (PR #188),
release v4.11.0, et **étape 1 de la tranche 2 de #154 livrée côté crate**
— `SurveyContext { site, sensor, interface }` dans le préambule SFMS,
`SFMS_VERSION` à 2, valeurs percent-encodées, lecture v1 inchangée
(PR #189, sonar-flows-core/sonar-flows-cli 0.6.0 publiées sur crates.io
le 06/08 par le tag `crates-v0.6.0`), puis re-vendorées dans le desktop
le même jour (PR #190) : l'application consomme désormais 0.6.0. Le
format est donc en place de bout en bout ; ce qui reste de la tranche 2
est le câblage fonctionnel, pas la dépendance. Gates vérifiées vertes
sur `ce770b2d` le 06/08 —
frontend 181 tests + ESLint + `vue-tsc`, desktop 199 tests, cœur partagé
91 tests, `fmt` et `clippy -D warnings` propres sur les trois.

Chemin critique vers la RC : #154 → #151 → #159/#160 → #136/#162 →
#94/#146.

## Ordre immédiat

1. Sprint actif (démarré le 06/08) : fiabilité prouvable —
   [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150)
   (comptabilité), puis
   [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151)
   (corpus/fuzzing) et
   [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136)
   (release immuable) — lots et avancement dans `sprint.md`.
2. Livré le 06/08 sur #154 : contexte de relevé automatique dans le
   préambule SFMS et manifest v2 du `.sonar` avec test de migration
   (PR #193, mergée). Reliquat reporté : `origin` par flux.
3. Ensuite [#160](https://github.com/Sonar-team/Sonar_desktop_app/issues/160),
   puis la chaîne de release : [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136)
   et [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162),
   puis [#94](https://github.com/Sonar-team/Sonar_desktop_app/issues/94)
   (signatures) et [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146)
   (E2E installateurs).

## Sprint actif — fiabilité prouvable et chaîne de release

Démarré le 06/08/2026. Lots et Definition of Done dans `sprint.md` :
comptabilité exhaustive des paquets (#150), corpus hostile et fuzzing (#151),
release immuable (#136).

**Arbitrage du 06/08** : recentrage sans retour arrière. La persistance
(#159 phase A) et la qualification automatique du relevé (#193, demande
utilisateur) restent dans main ; une extraction envisagée a été abandonnée le
jour même. Aucune nouvelle fonctionnalité tant que les trois lots ne sont pas
livrés. #191 (XLSX cœur + sigma.js) reste dormante en branche. Phases B/C de
#159 et reliquat #154 (`origin` par flux) reportés.

## Sprint précédent — fidélité des données et intégrité des sessions (reliquat)

Suivi : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165)

- [x] **P0** [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87) — reproduire l'import infini ou le fermer avec preuve et test *(fermée le 14/07, non reproductible avec preuves ; reliquat UI traité dans #161, fermée le 28/07)*
- [ ] **P0** [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) — supprimer les skips silencieux, puis construire le corpus complet *(skips supprimés le 14/07 — corpus nDPI + PCAPNG forgés ; oracle TShark livré via #168 le 23/07 ; reste le fuzzing et le multi-DLT PCAPNG)*
- [x] **P0** [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) — réserver atomiquement l'état `Importing` pendant toute conversion *(fait le 14/07)*
- [ ] **P0** [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) — détecter le DLT et comptabiliser exhaustivement les résultats de parsing *(l'essentiel est livré depuis le 15/07 : rapport qualité visible, identité RAW/SLL/SLL2 réimportable ; l'issue reste ouverte pour le reliquat — la fermer ou la découper)*
- [x] **P0** [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) — ne perdre aucun paquet accepté à l'arrêt ou au plafond de flux *(fait le 14/07)*
- [x] **P0** [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) — générer et tester le contrat IPC Rust ↔ TypeScript *(refermée le 15/07 après huit revues — voir sprint.md)*
- [ ] **P0** [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) — identité d'actif contextualisée par site, capteur, interface et VLAN *(tranche 1 — clé de nœud (vlan, ip), ids stables, anomalies IP dupliquée/multi-MAC — livrée le 04/08 via sonar-flows-core 0.5.0, PR #186/#187 ; tranche 2 étape 1 — `SurveyContext` et SFMS v2 — livrée le 06/08 via sonar-flows-core 0.6.0, PR #189 ; reste le câblage desktop : saisie au stop/save, manifest v2, `origin` par flux ; tranche 3 déplacée vers #164)*
- [ ] **P1 validation** [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) — revalider immédiatement les chemins espaces/Unicode *(backend testé le 14/07 ; restent Windows et front)*
- [ ] **P1** *(sans issue — à ouvrir)* — créer les fichiers de test (pcap simples forgés + matrices CSV attendues) et les tests d'intégration couvrant import pcap, conversion pcap → matrice, export et ré-import de matrice *(arborescence `src-tauri/test_files/pcaps/` créée le 16/07 ; l'oracle TShark #168 couvre une partie de l'exactitude depuis le 23/07)*

La Definition of Done détaillée est dans `sprint.md` et dans l'issue #165.

## Sprint suivant — sessions et parcours produit terminés

- [ ] **P0** [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159) — projets persistants, autosave, récupération et manifest de preuve
- [ ] **P0** [#160](https://github.com/Sonar-team/Sonar_desktop_app/issues/160) — matrice de flux de production dans le parcours principal
- [x] **P0** [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) — intégrité frontend : doublons, imports bloqués et erreurs invisibles *(fermée le 28/07 ; les 6 findings hauts de l'audit frontend du 01/08 — erreurs encore avalées — corrigés le 02/08, `b3d42a07`)*
- [ ] **P1** [#111](https://github.com/Sonar-team/Sonar_desktop_app/issues/111) — fiabiliser tous les parcours sauvegarde/export
- [ ] **P1** [#102](https://github.com/Sonar-team/Sonar_desktop_app/issues/102) — durcir le support bundle ZIP *(export ZIP des logs livré le 02/08, `ff653d82` ; restent confidentialité, manifest et tests multi-OS)*
- [x] **P1** [#145](https://github.com/Sonar-team/Sonar_desktop_app/issues/145) — supprimer ou migrer les routes et vues héritées *(fermée le 29/07)*
- [ ] **P1** [#144](https://github.com/Sonar-team/Sonar_desktop_app/issues/144) — rendre les parcours principaux conformes WCAG 2.2 AA *(reliquat a11y de l'audit du 01/08 : `role="dialog"`/`aria-modal`/Échap manquants sur 5 modales)*

## Sprint suivant — distribution professionnelle

- [ ] **P0** [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136) — release immuable : construire une fois, tester puis publier *(rouverte — la release atomique du 11/07 ne suffisait pas)*
- [ ] **P0** [#94](https://github.com/Sonar-team/Sonar_desktop_app/issues/94) — Authenticode, Developer ID, notarisation et Apple Silicon
- [ ] **P0** [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146) — E2E Tauri et installateurs réellement testés sur chaque OS
  - [ ] Ajouter Cypress Component + E2E navigateur pour les parcours Vue/Vite
    avec les API Tauri simulées ; conserver WebdriverIO/Tauri pour le binaire,
    l’IPC Rust et les validations natives par OS.
- [ ] **P0** [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) — quality gates et scans bloquants sur toute release taggée
- [x] [#175](https://github.com/Sonar-team/Sonar_desktop_app/issues/175) — le .deb ne déclare pas libpcap dans `Depends:` *(corrigé le 04/08, PR #185 — vérifié en conteneur Debian trixie propre)*
- [ ] **P1** [#96](https://github.com/Sonar-team/Sonar_desktop_app/issues/96) — modèle de menace, preuve de passivité et durcissement runtime
- [ ] **P1** [#143](https://github.com/Sonar-team/Sonar_desktop_app/issues/143) — moindre privilège Tauri, chemins validés et helper de capture
- [ ] **P1** [#163](https://github.com/Sonar-team/Sonar_desktop_app/issues/163) — documentation et support d'une distribution professionnelle
- [x] **P2** [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138) — Npcap prérequis externe — détection installateur et suivi Nmap *(fermée le 20/07 : Npcap reste externe, jamais redistribué)*

## Différenciation Pro — après fermeture des P0

- [ ] **P2** [#164](https://github.com/Sonar-team/Sonar_desktop_app/issues/164) — baseline/diff, inventaire d'actifs, rapports attestables et spécification SFMS
- [ ] **P2** [#156](https://github.com/Sonar-team/Sonar_desktop_app/issues/156) — arguments de session du desktop pour orchestration et recette
- [ ] **P1** [#132](https://github.com/Sonar-team/Sonar_desktop_app/issues/132) — performance de capture sous forte charge, après fidélité

## Backlog GitHub historique à requalifier

Ces tickets restent ouverts avec une priorité P1 à P3. Leur périmètre n'a
pas été réécrit en détail et doit être revalidé avant intégration dans un
sprint.

- [ ] **P1** [#98](https://github.com/Sonar-team/Sonar_desktop_app/issues/98) — VAE SONAR, gate de Release Candidate
- [ ] **P2** [#107](https://github.com/Sonar-team/Sonar_desktop_app/issues/107) — paquet Debian non reproductible
- [ ] **P2** [#119](https://github.com/Sonar-team/Sonar_desktop_app/issues/119) — reproductibilité NSIS
- [ ] **P2** [#120](https://github.com/Sonar-team/Sonar_desktop_app/issues/120) — reproductibilité DMG
- [ ] **P2** [#174](https://github.com/Sonar-team/Sonar_desktop_app/issues/174) — repro conteneur : SDK Windows (xwin) et preuve inter-machines
- [ ] **P2** [#101](https://github.com/Sonar-team/Sonar_desktop_app/issues/101) — refonte visuelle des icônes
- [ ] **P3** [#91](https://github.com/Sonar-team/Sonar_desktop_app/issues/91) — homogénéisation visuelle des sous-menus

## Réalisé récemment

- Hors issues : `SurveyContext` dans le préambule SFMS et `SFMS_VERSION` 2 — sonar-flows-core / sonar-flows-cli **0.6.0** publiées sur crates.io *(PR #189, tag `crates-v0.6.0`, 06/08)*, re-vendorées dans le desktop le même jour *(PR #190)*. Étape 1 de la tranche 2 de #154.
- Hors issues : scénario E2E X11 resynchronisé avec l'application — boutons, ZIP, confirmation *(PR #188, 05/08)*
- [x] [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) — intégrité du cycle frontend *(28/07)* + correction des 6 findings hauts de l'audit du 01/08 *(02/08)*
- [x] [#145](https://github.com/Sonar-team/Sonar_desktop_app/issues/145) — routes et vues héritées supprimées *(29/07)*
- [x] [#97](https://github.com/Sonar-team/Sonar_desktop_app/issues/97) — validation Windows 11 *(29/07)*
- [x] [#89](https://github.com/Sonar-team/Sonar_desktop_app/issues/89) / [#90](https://github.com/Sonar-team/Sonar_desktop_app/issues/90) / [#92](https://github.com/Sonar-team/Sonar_desktop_app/issues/92) — À propos, filtres cohérents, légendes *(29/07)*
- [x] [#118](https://github.com/Sonar-team/Sonar_desktop_app/issues/118) / [#124](https://github.com/Sonar-team/Sonar_desktop_app/issues/124) / [#133](https://github.com/Sonar-team/Sonar_desktop_app/issues/133) / [#171](https://github.com/Sonar-team/Sonar_desktop_app/issues/171) — MSI, cargo-deny, crates.io, Dependabot glib *(29/07)*
- [x] [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166) — interblocage démarrages/arrêts concurrents *(24/07)*
- [x] [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167) — imports vides et resets concurrents refusés sans perdre la session *(23/07)*
- [x] [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168) — exactitude PCAP → matrice prouvée par oracle TShark *(23/07)*
- [x] [#169](https://github.com/Sonar-team/Sonar_desktop_app/issues/169) — releases historiques contenant Npcap assainies *(27/07)*
- [x] [#109](https://github.com/Sonar-team/Sonar_desktop_app/issues/109) / [#112](https://github.com/Sonar-team/Sonar_desktop_app/issues/112) / [#121](https://github.com/Sonar-team/Sonar_desktop_app/issues/121) — ConfigPanel typé, logs console retirés, snapshots APT durcis *(27/07)*
- [x] [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138) — décision Npcap : prérequis externe, détection/redirection *(20/07)*
- [x] [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) — contrat IPC généré Rust ↔ TypeScript *(15/07)*
- [x] [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) / [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) — atomicité import, drainage arrêt/plafond *(14/07)*
- [x] [#135](https://github.com/Sonar-team/Sonar_desktop_app/issues/135) / [#137](https://github.com/Sonar-team/Sonar_desktop_app/issues/137) / [#140](https://github.com/Sonar-team/Sonar_desktop_app/issues/140) / [#141](https://github.com/Sonar-team/Sonar_desktop_app/issues/141) / [#147](https://github.com/Sonar-team/Sonar_desktop_app/issues/147) / [#148](https://github.com/Sonar-team/Sonar_desktop_app/issues/148) / [#149](https://github.com/Sonar-team/Sonar_desktop_app/issues/149) / [#152](https://github.com/Sonar-team/Sonar_desktop_app/issues/152) / [#153](https://github.com/Sonar-team/Sonar_desktop_app/issues/153) / [#155](https://github.com/Sonar-team/Sonar_desktop_app/issues/155) / [#157](https://github.com/Sonar-team/Sonar_desktop_app/issues/157) — campagne du 11/07 *(11-12/07)*
- Hors issues : packet_parser 9.0.0 intégré via sonar-flows-core 0.4.0, vendoré et vetté *(PR #183, 02/08)*

## Règles de sortie de bêta

- aucun P0 ouvert avant la Release Candidate ;
- aucun P1 ouvert avant la 1.0 Pro, sauf dérogation écrite et limitée ;
- chaque paquet lu est classé ou compté comme perdu avec une raison ;
- aucun travail utilisateur n'est perdu sur stop, fermeture ou crash ;
- les parcours capture/import/matrice/graphe/labels/export passent en E2E ;
- les installateurs sont signés, notarifiés et testés sur machine propre ;
- les limites, prérequis et données sensibles sont documentés.
