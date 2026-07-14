# Sprint P0 — Fidélité des données et intégrité des sessions

> Statut : actif
> Dernière revue : 14/07/2026 (v4.5.0 : DLT réel, préambule #SFMS, rejet inter-DLT)
> Source : audit bêta → pro du 13/07/2026
> Suivi GitHub : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165)
> Priorisation :
> [project_management/priorisation_beta_to_pro.md](project_management/priorisation_beta_to_pro.md)

## Objectif

Garantir qu'aucun paquet, flux ou état de session ne puisse être perdu,
ignoré, fusionné ou remplacé silencieusement.

## Phase 0 — rendre les défauts observables

1. [ ] [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87) :
   reproduire l'import infini ou le fermer avec preuve et test.
2. [ ] [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) :
   supprimer tout succès obtenu en sautant une fixture absente.
3. [ ] [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) :
   revalider espaces/Unicode et créer les tests nécessaires.

## Phase 1 — atomicité et comptabilité

4. [x] [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) :
   réserver `Importing` pendant toute la conversion. *(14/07 : phase
   `Importing` + guard RAII, tests de course déterministes.)*
5. [ ] [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) :
   définir le résultat canonique de parsing et le rapport qualité.
   *(14/07 : DLT réel branché partout, refus avant mutation, préambule
   `#SFMS`, fusion inter-DLT rejetée — reste le rapport qualité UI/export
   et le réimport SLL exact, cf. commentaire du 14/07.)*
6. [ ] [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) :
   drainer arrêt et plafond avec des compteurs exacts.
7. [ ] [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) :
   compléter multi-DLT, malformé, PCAPNG et fuzzing.

## Phase 2 — intégration et identité

8. [ ] [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) :
   générer et tester le contrat IPC Rust → TypeScript.
9. [ ] [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) :
   stabiliser l'identité d'actif contextualisée.

## Livrables

- classification canonique partagée cœur, CLI et desktop ;
- égalité vérifiable entre paquets lus et toutes les catégories ;
- pertes noyau, interface et application distinguées ;
- import protégé contre toute capture/reset concurrent ;
- arrêt sans paquet accepté abandonné silencieusement ;
- corpus assaini ou généré déterministement ;
- contrat IPC généré et exhaustif ;
- identité tenant compte du projet/site, capteur, interface et VLAN.

## Travail parallèle autorisé

- [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) :
  double batch, déduplication des fichiers et déverrouillage `finally` ;
- [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) :
  workflow qualité commun aux PR et releases ;
- conception de [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159),
  sans figer son schéma avant #154.

## Definition of Done

- [ ] Chaque paquet lu appartient à une catégorie explicite.
- [x] Un DLT non supporté échoue avant toute mutation de l'état. *(v4.5.0)*
- [x] Une capture ne peut pas démarrer pendant un import. *(#139, 14/07)*
- [ ] Stop et limite de flux drainent ou comptent la perte exacte.
- [ ] Aucun test critique ne dépend silencieusement d'un fichier local.
- [ ] Le rapport final traverse un IPC généré, est visible et exportable.
- [ ] Deux actifs de même IP sur des VLAN/sites distincts ne sont pas fusionnés.
- [ ] Les courses et chemins d'arrêt ont des tests déterministes.
- [ ] Typecheck, tests, builds, fmt et Clippy strict sont verts.
- [ ] Les DLT supportés et limites sont documentés.

## Hors périmètre

- produit et persistance : #159, #160, #161 ;
- distribution : #94, #138, #146, #162 ;
- documentation/support : #163 ;
- différenciation : #164.
