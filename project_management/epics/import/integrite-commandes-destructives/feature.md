# Feature : commandes d'import et reset non destructifs

> Epic : import — Statut : sprint actif (P0)
> Issue : [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167)

Refuser les listes d'import vides avant tout événement ou mutation et empêcher
un reset concurrent de contourner la réservation `Importing`. Toute erreur
conserve matrice, graphe, labels et métadonnées de la session courante.

## User stories

- [ ] US-01 — un import vide ne remplace jamais mon travail par un état vide
- [ ] US-02 — un reset concurrent est refusé avec une erreur explicite
