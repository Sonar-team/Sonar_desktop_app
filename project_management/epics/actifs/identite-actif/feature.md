# Feature : identité d'actif contextualisée

> Epic : actifs — Statut : sprint actif (P0)
> Issue : [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154)

Clé d'actif stable tenant compte du projet/site, du capteur, de l'interface
et du VLAN : deux IP identiques dans des contextes différents ne doivent
jamais être fusionnées implicitement. Les décisions de corrélation sont
visibles et réversibles ; la migration des exports existants est définie.

## User stories

- [ ] [US-01 — ne pas fusionner deux actifs de VLAN différents](us-01-pas-de-fusion-inter-vlan.md)
- [ ] US-02 — à rédiger : voir et annuler une décision de corrélation d'actifs
- [ ] [US-03 — nœuds et arêtes du graphe sans adresse de liaison](us-03-noeud-sans-adresse-liaison.md)
  (arbitré le 14/07/2026, à implémenter avec le branchement DLT #150)
