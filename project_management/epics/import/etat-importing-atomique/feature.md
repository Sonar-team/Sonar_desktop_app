# Feature : état Importing réservé atomiquement

> Epic : import — Statut : lot livré le 14/07/2026 (P0 fermé)
> Issue : [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139)

Réserver la transition `Idle → Importing → Idle` pendant toute la
conversion : capture et second import refusés pendant l'import, nettoyage
garanti sur succès et erreur.

Le reset concurrent, la génération d'opération et le refus des listes vides
sont transférés vers
[#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167).

## User stories

- [x] US-01 — une capture ou un second import ne peut pas remplacer une
  conversion déjà réservée
- [ ] US-02 — un reset ou un import vide ne peut pas muter la session (#167)
