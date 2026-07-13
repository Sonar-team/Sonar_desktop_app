# Feature : état Importing réservé atomiquement

> Epic : import — Statut : sprint actif (P0)
> Issue : [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139)

Réserver la transition `Idle → Importing → Idle` pendant toute la
conversion : capture, second import et reset refusés pendant l'import,
identifiant d'opération contrôlé au commit, nettoyage garanti sur succès,
annulation et erreur.

## User stories

- [ ] US-01 — à rédiger : impossible de corrompre une session en lançant deux
  opérations concurrentes
