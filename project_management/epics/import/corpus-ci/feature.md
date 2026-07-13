# Feature : corpus PCAP/PCAPNG de qualification CI

> Epic : import — Statut : sprint actif (P0)
> Issue : [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151)

Corpus assaini ou généré déterministement couvrant Ethernet, VLAN, tunnels,
SLL/SLL2, RAW, loopback, PCAPNG multi-interface, fichiers tronqués et trames
malformées. Aucun test ne doit être silencieusement sauté ni dépendre d'un
fichier local ; seeds de régression versionnés.

## User stories

- [ ] US-01 — à rédiger : chaque DLT supporté est prouvé par un test CI
  reproductible
