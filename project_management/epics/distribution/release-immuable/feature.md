# Feature : release immuable construite une seule fois

> Epic : distribution — Statut : planifié P0 (issue rouverte)
> Issue : [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136)

Construire chaque artefact une seule fois, puis tester, scanner, signer,
attester et publier exactement les mêmes octets. Les jobs de build ne
possèdent aucun droit de publication et un rerun ne remplace jamais les
artefacts d'une release publique.

## User stories

- [ ] US-01 — vérifier que l'installateur publié contient le binaire testé
- [ ] US-02 — relancer un workflow sans modifier une release publique
