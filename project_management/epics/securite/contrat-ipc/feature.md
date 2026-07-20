# Feature : contrat IPC Rust ↔ TypeScript

> Epic : securite — Statut : livré le 15/07/2026 (P0 fermé)
> Issue : [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142)

Générer les types TypeScript depuis les définitions Rust et tester le
contrat de bout en bout : plus aucune dérive silencieuse entre commandes,
events et stores.

## User stories

- [x] US-01 — une modification de contrat Rust incompatible casse la CI avant
  d'atteindre l'utilisateur
