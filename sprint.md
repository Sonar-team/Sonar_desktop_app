# Sprint P0 — Fiabilité prouvable et chaîne de release

> Statut : actif (démarré le 06/08/2026)
> Suivi GitHub : [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150),
> [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151),
> [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136)
> Sprint précédent (sessions persistantes, #159) : phase A livrée le 04/08
> (PR #184) et **conservée dans main** ; phases B et C reportées — voir
> « Arbitrage » ci-dessous. Historique détaillé dans le sprint.md antérieur
> (git) et l'issue #159.

## Objectif

Une application dont l'utilisateur peut **prouver** la fiabilité, et un binaire
dont on peut **prouver** l'origine. Rien d'autre dans ce sprint.

Formulé par Cyprien le 06/08/2026 : « avoir une app qui prouve bien aux
utilisateurs que la matrice est fiable, et SONAR aussi, et avoir un build
reproductible pour la sécu ».

## Arbitrage du 06/08/2026 — recentrage sans retour arrière

Une extraction de la persistance (#159) hors de `main` a été envisagée puis
**abandonnée le jour même** : la qualification du relevé (#193, mergée) répond
à une demande utilisateur réelle et son schéma v2 vit dans le module projet.
`main` garde donc la persistance (phase A), le contexte de relevé automatique
et les corrections de fuite d'étiquette.

Le recentrage porte sur la suite : **aucune nouvelle fonctionnalité tant que
les trois lots ci-dessous ne sont pas livrés.** Restent hors de main, en
branche : #191 (XLSX dans le cœur + rendu sigma.js, non vetté, sans test).
Reportées : phases B/C de #159 (v2 étendu, manifest de preuve signé),
tranche restante de #154 (`origin` par flux), #164.

## Lot 1 — prouver que la matrice est fiable (#150)

1. [ ] Équation de comptabilité vérifiée dans le cœur :
   `lus = décodés + DLT non supportés + tronqués + erreurs de lecture`.
2. [ ] Le desktop expose ces catégories **séparément** dans les événements
   finaux, pas un total agrégé.
3. [ ] Le bilan est visible dans l'UI **et** embarqué dans les exports et le
   projet `.sonar`.
4. [ ] Tests exhaustifs : Ethernet, SLL/SLL2, RAW, loopback, multi-interface,
   fichiers tronqués.

## Lot 2 — prouver que SONAR ne ment pas sur des entrées hostiles (#151)

5. [ ] Corpus qualifié : Ethernet, VLAN, tunnels, SLL/SLL2, RAW, loopback,
   PCAPNG multi-interface.
6. [ ] Fichiers tronqués, longueurs incohérentes, trames malformées.
7. [ ] Fuzz/property tests du parseur et de l'import, budget CI borné.
8. [ ] Les seeds de régression issus du fuzzing rejoignent le corpus.
9. [ ] Corpus audité comme dépourvu de données réseau sensibles.

## Lot 3 — prouver l'origine du binaire (#136)

10. [ ] Construire une fois, tester **ce** binaire, publier **exactement**
    celui-ci — la release atomique du 11/07 ne suffisait pas, d'où la
    réouverture de l'issue.
11. [ ] Quality gates et scans bloquants sur toute release taguée (#162).

## Definition of Done

- [ ] Chaque paquet lu appartient à une catégorie fine explicite, et
  l'utilisateur peut la lire sans ouvrir un fichier de log.
- [ ] Un fichier hostile ne produit ni panic, ni compteur faux, ni succès
  silencieux.
- [ ] Aucun test ne réussit en sautant une fixture absente.
- [ ] L'artefact publié est bit-à-bit celui qui a été testé.
- [ ] Typecheck, tests, builds, fmt et Clippy verts sur le SHA publié.

## Hors périmètre

- phases B/C de #159 (schéma v2 étendu, manifest de preuve signé) ;
- reliquat #154 (`origin` par flux, fusion multi-sites) et #164 ;
- #191 (XLSX dans le cœur, rendu sigma.js) — dormante en branche ;
- toute fonctionnalité d'interface qui ne sert pas une preuve.
