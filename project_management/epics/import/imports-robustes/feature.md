# Feature : imports robustes (chemins et fichiers difficiles)

> Epic : import — Statut : partiellement livré
> Issues : [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87) (P0),
> [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) (P1)

Revalider les imports PCAP qui ne terminent jamais (#87) et accepter les
chemins avec espaces, Unicode et caractères spéciaux (#88).

## User stories

- [x] US-01 — tout import backend se termine ou échoue avec une raison (#87)
- [ ] US-02 — importer un fichier quel que soit son nom ou chemin
  (backend validé ; Windows et frontend restent dans #88)
