# Sprint P0 — Fiabilité prouvable et chaîne de release

> Statut : actif (démarré le 06/08/2026) — lots 1 et 2 pour l'essentiel
> livrés le jour même, voir les cases ci-dessous
> Suivi GitHub : [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151),
> [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136)
> (#150 fermée le 06/08, reliquat export transféré vers #111)
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
Reportées vers #164 (transferts actés par le triage du 06/08, qui a fermé
#88, #150, #154, #159, #165 et, en « not planned », #91, #101, #156) :
manifest de preuve signé, `origin` par flux, corrélation d'actifs.

## Lot 1 — prouver que la matrice est fiable (#150, fermée le 06/08)

1. [x] Équation de comptabilité vraie **par construction** dans le cœur :
   `lus = décodés + tronqués + DLT non supportés + malformés` — le total est
   dérivé de la somme des catégories (0.7.0, PR #195).
2. [x] Le desktop expose les catégories séparément dans les événements
   finaux — import (`Finished`) et capture live (`Stats`), drainage compris ;
   classification unique du cœur (PR #196, #197).
3. [x] Bilan visible dans l'UI (badge d'import et compteur live, équation en
   infobulle) — l'embarquement dans les exports est transféré vers #111.
4. [x] Tests : corpus multi-DLT + oracles TShark sur les catégories, preuve
   terrain tronqué/malformé/décodé forgée, frontière loopback, PCAPNG
   multi-interface, fichiers tronqués fatals.

## Lot 2 — prouver que SONAR ne ment pas sur des entrées hostiles (#151)

5. [x] Corpus qualifié : Ethernet, VLAN, tunnels, SLL/SLL2, RAW, **loopback**
   (refus explicite testé), PCAPNG multi-interface (forgés en tests).
6. [ ] Fichiers tronqués et trames malformées : preuve terrain livrée,
   couverture **exhaustive** (longueurs incohérentes par champ) restante.
7. [x] Fuzzing en CI à budget borné : job `fuzz_smoke`, cibles `pcap_reader`
   et `matrix_reader`, corpus dérivé des fixtures versionnées. Première
   campagne : **un panic réel trouvé et corrigé** (timestamp pcapng négatif
   → débordement `SystemTime`, 0.8.0) — menace listée par #96.
8. [x] Seeds de régression versionnées (`fuzz/regressions/<cible>/`),
   recopiées dans le corpus par `prepare-fuzz-corpus.sh`.
9. [ ] Corpus audité comme dépourvu de données réseau sensibles — le README
   admet des adresses publiques dans les captures SLL/SLL2 réelles.

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
