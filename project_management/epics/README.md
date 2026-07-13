# Epics SONAR — catalogue produit

> Convention : un dossier par **epic** (`epic.md`), un sous-dossier par
> **feature** (`feature.md`), et une **user story** par fichier
> (`us-NN-slug.md`) dans le dossier de sa feature.
> Les issues GitHub restent la source de vérité ; `todo.md` donne l'ordre
> d'exécution, `Roadmap.md` la trajectoire.

## Epics

| Epic | Périmètre | Issues principales |
|---|---|---|
| [capture](capture/epic.md) | Capture live, filtres, arrêt fidèle, performance | #158, #90, #132, #156 |
| [import](import/epic.md) | Import PCAP/PCAPNG, parsing, corpus CI | #150, #139, #151, #87, #88 |
| [visualisation](visualisation/epic.md) | Matrice de flux, graphe, tri/édition/fusion | #160, #92 |
| [actifs](actifs/epic.md) | Identité d'actif, labels, inventaire, baseline | #154, #164 |
| [projets-sessions](projets-sessions/epic.md) | Persistance, autosave, récupération, manifest | #159 |
| [exports-rapports](exports-rapports/epic.md) | Sauvegarde/export, support bundle, rapports | #111, #102, #164 |
| [interface](interface/epic.md) | Intégrité frontend, accessibilité, finitions UI | #161, #144, #145 |
| [distribution](distribution/epic.md) | Signatures, installateurs, gates, reproductibilité | #94, #146, #162 |
| [securite](securite/epic.md) | Modèle de menace, moindre privilège, contrat IPC | #96, #143, #142 |

## Statuts

- `sprint actif` — dans le sprint en cours (`sprint.md`, #165) ;
- `planifié` — ordonné dans une phase de `Roadmap.md` ;
- `backlog` — ouvert mais non ordonné ;
- `à requalifier` — hérité, priorité à revoir avant intégration ;
- `livré` — terminé, conservé pour référence.
