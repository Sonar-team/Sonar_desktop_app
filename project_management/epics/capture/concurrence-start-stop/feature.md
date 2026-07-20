# Feature : concurrence start/stop sans interblocage

> Epic : capture — Statut : sprint actif (P0)
> Issue : [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166)

Garantir qu'un démarrage et un arrêt concurrents ne puissent jamais former un
cycle de verrous entre `CaptureState`, la matrice et les threads joints.
L'arrêt rend toujours une phase cohérente et conserve le drainage de #158.

## User stories

- [ ] US-01 — démarrer et arrêter concurremment sans gel de l'application
- [ ] US-02 — diagnostiquer toute attente dépassant la borne du test
