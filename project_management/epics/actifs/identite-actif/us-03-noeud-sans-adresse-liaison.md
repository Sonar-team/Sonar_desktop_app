# US-03 — Nœuds et arêtes du graphe sans adresse de liaison

> Arbitrage du 14/07/2026 (migration packet_parser 7, multi-LINKTYPE #150).
> Contexte : depuis packet_parser 7, la couche liaison est fidèle au
> LINKTYPE — RAW IP n'a aucune adresse, Linux SLL/SLL2 n'a qu'une adresse
> source. Le graphe ne peut plus supposer deux MAC par paquet.

En tant qu'auditeur ICS, je veux que le graphe représente chaque équipement
dès qu'une identité réelle est observée (IP, sinon adresse de liaison), sans
jamais inventer d'identité pour combler un champ absent, afin que
l'inventaire reflète exactement ce que la capture prouve.

## Règle d'identité

- **Un nœud existe dès qu'un côté du paquet a une identité** : IP en
  priorité (chemin L3 actuel), sinon adresse de liaison (repli L2). Une
  capture SLL avec seulement une adresse source produit le nœud source,
  même sans arête.
- **Une arête n'existe que si les deux extrémités ont une identité.** Pas
  de pseudo-nœud « destination inconnue » : toutes les destinations
  inconnues convergeraient vers le même nœud fictif.
- **Jamais d'identité inventée** : pas de MAC nulle (`00:00:…`), pas de
  clé vide (`mac:`), pas d'EtherType fabriqué. Une adresse absente est
  absente (chaîne vide côté texte, cf. `sonar_flows_core::link::LinkView`).
- **Rien de silencieusement perdu** : un paquet sans aucune identité des
  deux côtés ne crée ni nœud ni arête, mais reste compté dans la matrice
  et dans la comptabilité de parsing (#150). L'écart nœuds/arêtes doit se
  lire dans les compteurs, pas se deviner.

## Critères d'acceptation

- une capture RAW IP produit des nœuds L3 (IP) sans attribut MAC, jamais
  de nœud `00:00:00:00:00:00` ;
- un paquet non-IP sur SLL (ex. ARP) produit le nœud de son adresse
  source, sans arête vers un nœud fictif ;
- un paquet sans IP ni adresse de liaison n'apparaît pas dans le graphe
  mais est compté (matrice + compteurs de parsing) ;
- le repli L2 du graphe (`GraphData::add_packet_flow`) n'utilise jamais
  une clé de nœud vide ;
- tests unitaires du graphe couvrant les trois cas (RAW IP, SLL
  source-seule, aucune identité).
