# Feature : quality gates de release taguée

> Epic : distribution — Statut : en cours (partiel, P0)
> Issue : [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162)

Les gates ESLint et Prettier sont en place depuis le 17/07/2026. Le blocage
complet d'une release reste ouvert, notamment sur l'alignement des versions et
la garantie « build once, test, publish same bytes » suivie par
[#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136).

Toute release taguée passe des quality gates et scans bloquants (tests,
lints, audit de dépendances, SBOM) avec couverture complète — aucune étape
optionnelle ou silencieusement sautée.

## User stories

- [ ] US-01 — à rédiger : impossible de publier une release qui n'a pas passé
  tous les gates
- [ ] US-02 — aligner les versions applicatives et qualifier le SHA publié
- [ ] US-03 — publier exactement les artefacts testés, sans reconstruction ni
  écrasement silencieux
