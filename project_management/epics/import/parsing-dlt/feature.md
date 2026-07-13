# Feature : fidélité du parsing et détection du DLT

> Epic : import — Statut : sprint actif (P0)
> Issue : [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150)

Détecter le DLT du fichier, rejeter explicitement les DLT non supportés
avant toute mutation d'état, et classer chaque paquet lu dans une catégorie
de résultat canonique partagée par le cœur, la CLI et le desktop. La partie
amont vit dans la crate `packet_parser` (à signaler, jamais à éditer dans le
vendor).

## User stories

- [ ] US-01 — à rédiger : voir un bilan « paquets lus = somme des catégories »
- [ ] US-02 — à rédiger : être prévenu avant import qu'un DLT n'est pas supporté
