# Ultimate Ethernet sample

Cette fixture est un échantillon reproductible de
`The-Ultimate-PCAP.pcapng`. Le fichier source ne peut pas être donné tel quel
à `libpcap` : ses 313 interfaces mélangent plusieurs encapsulations et
plusieurs `snaplen`. Sur Linux/libpcap 1.10, l'ouverture échoue explicitement
avec `an interface has a snapshot length 8192 different from the snapshot
length of the first interface`.

## Provenance

- source SHA-256 :
  `ecbca543fbe011a85c8876420fcb2c244b9290ad653d13cd1a574158167073f4` ;
- source : 51 328 trames, dont 39 557 Ethernet, 10 667 IEEE 802.3br
  mPackets, 1 067 Linux SLL et 37 PPP ;
- oracle : TShark 4.6.6, résolution de noms désactivée, fuseau UTC ;
- échantillonnage : les trois premières trames Ethernet de chaque valeur
  distincte de `_ws.col.Protocol`, dans l'ordre du fichier source ;
- normalisation : écriture PCAPNG filtrée, puis `editcap -F pcap -T ether` ;
- fixture SHA-256 :
  `91252b10568ae9faed0c7262aa16c7c824643d478a392689345e1f63a43f0dde`.

La fixture résultante couvre 112 libellés de protocole TShark avec 328 trames
et 96 622 octets. Le rapport `ultimate_ethernet_sample.tshark.md` contient
l'inventaire paquet par paquet, la hiérarchie des protocoles, les conversations
et les endpoints. Sa vérification agrégée donne **PASS** face au snapshot SFMS :
216 lignes, 328 paquets et 96 622 octets.

Le test Rust compare ensuite les 216 lignes SFMS complètes et vérifie un second
export octet par octet. TShark et SONAR n'ont pas la même taxonomie applicative ;
le CSV fige donc aussi la projection SFMS détaillée attendue.

## Comparaison différentielle TShark ↔ SFMS

`ultimate_ethernet_sample.flows.tsv` est un second oracle, indépendant du CSV
SONAR. Il est régénéré directement depuis les champs numériques TShark avec :

```shell
python3 script/pcap/generate-common-flows.py \
  src-tauri/test_files/pcaps/import/ultimate_ethernet_sample/ultimate_ethernet_sample.pcap \
  src-tauri/test_files/pcaps/import/ultimate_ethernet_sample/ultimate_ethernet_sample.flows.tsv
```

La clé directionnelle commune compare les MAC, le VLAN, l'EtherType, les IP,
le numéro de protocole IP et les ports. Les statistiques comparent le nombre de
trames, la somme de `frame.len` et le dernier timestamp à la microseconde. Les
lignes SONAR qui ne diffèrent que par le protocole applicatif sont fusionnées
avant comparaison.

L'oracle couvre 313 trames, 94 091 octets et 202 flux, tous égaux à la matrice.
Quinze trames sont hors du domaine commun : longueurs IEEE 802.3 et VLAN dont
le payload EtherType n'est pas représentable sans perte dans SFMS v1. Cette
exclusion est effectuée par des règles structurelles explicites dans le
générateur, jamais à partir du résultat SONAR.
