# SFMS — SONAR Flow Matrix Standard (v1)

> Description du format des matrices de flux produites et consommées par
> SONAR (desktop et `sonar-cli`). Ce document décrit le format **tel
> qu'implémenté** dans `sonar-flows-core` ; la spécification formelle
> (schéma JSON + validateur de conformité, mapping IPFIX) est un axe de la
> [vision](./VISION.md) (Axe D). Le modèle des tunnels est détaillé dans
> [`TUNNELS.md`](./TUNNELS.md).

## Principes

- **Une ligne = une conversation observée**, au niveau paquet : rien n'est
  agrégé, inféré ou interprété au-delà de ce qui a été vu sur le réseau.
- **Un relevé = un réseau = un type de liaison (DLT)** : une matrice porte
  le DLT de sa source ; la fusion de sources de DLT différents est refusée
  explicitement.
- **Déterminisme** : deux exports de la même matrice sont identiques octet
  pour octet. Le préambule ne porte volontairement pas de date d'export.
- **Aller-retour sans perte** : une matrice exportée puis réimportée
  reconstruit exactement les mêmes conversations (identité SFMS préservée,
  y compris pour RAW, SLL et SLL2).

## Préambule

La première ligne d'un export CSV est une ligne de métadonnées :

```
#SFMS version=1 dlt=ETHERNET
```

- `version` : version du format de préambule (actuellement `1`).
- `dlt` : type de liaison du relevé — mnémonique `LINKTYPE_*` connu
  (`ETHERNET`, `RAW`, `LINUX_SLL`, `LINUX_SLL2`, `IEEE802_11`) ou valeur
  numérique tcpdump pour les autres. Jamais de nom inventé.
- Les clés inconnues sont tolérées à l'import (extensions futures) ; un
  préambule présent mais illisible est une erreur, pas un avertissement.
- Un fichier **sans** préambule (export antérieur au format) est importé
  avec un DLT implicite Ethernet.

## Colonnes

| Colonne | Type | Sémantique |
|---|---|---|
| `mac_source` | chaîne | Adresse MAC source de la conversation. |
| `mac_destination` | chaîne | Adresse MAC destination. |
| `vlan_id` | entier ou vide | Identifiant 802.1Q si observé. |
| `protocol_data_link` | chaîne | Protocole de couche 2 (ex. `Ipv4`, `Arp`). |
| `ip_source` | IP ou vide | IP source ; vide pour un flux sans couche 3. |
| `ip_source_type` | chaîne | Qualification de l'adresse : `Private`, `Public`, `Multicast`, `Loopback`, `Apipa`, `LinkLocal`, `Ula`, `Documentation`, `Unknown`. |
| `label_source` | chaîne ou vide | Label de l'équipement source (référentiel de labels). |
| `ip_destination` | IP ou vide | IP destination. |
| `ip_destination_type` | chaîne | Même qualification que `ip_source_type`. |
| `label_destination` | chaîne ou vide | Label de l'équipement destination. |
| `port_source` | entier ou vide | Port source (transport). |
| `port_destination` | entier ou vide | Port destination. |
| `protocol_transport` | chaîne ou vide | Protocole de transport (`TCP`, `UDP`, …). |
| `application_protocol` | chaîne ou vide | Protocole applicatif reconnu par le parsing (dont OT/ICS : Modbus, Profinet, OPC UA, S7Comm, SNMP, EtherNet/IP…). |
| `count` | entier | Nombre de paquets observés pour cette conversation. |
| `total_bytes` | entier | Total des octets observés. |
| `last_seen` | date | Dernière observation, format `AAAA-MM-JJ HH:MM:SS[.µµµµµµ] UTC`. |
| `encap_id` | chaîne ou vide | Tunnels traversés (extension SFMS, voir ci-dessous). |
| `origin` | chaîne ou vide | Provenance de la ligne (voir ci-dessous). |

Les colonnes `encap_id` et `origin` sont optionnelles à l'import
(compatibilité avec les matrices exportées avant leur introduction).

### `encap_id` — tunnels (extension SFMS)

- vide : flux jamais observé dans un tunnel ;
- `id` (16 caractères hexadécimaux) : un seul tunnel portant tous les
  paquets de la ligne ;
- `id:n|id:n|…` : comptes ventilés par tunnel quand un flux apparaît dans
  plusieurs tunnels ou partiellement hors tunnel.

L'identifiant est partagé entre la ligne externe (le tunnel, ex. CAPWAP) et
ses lignes internes (le trafic décapsulé), dans les deux sens. La somme des
`n` d'un tunnel sur les lignes filles égale le compteur de sa ligne externe.
Détails et invariants : [`TUNNELS.md`](./TUNNELS.md).

### `origin` — provenance

Noms des fichiers ayant observé la conversation, joints par `|` quand la
ligne résulte d'une fusion multi-fichiers (ex. `site-a.csv|site-b.csv`).
Vide pour une capture live ou un import PCAP direct. Limite connue : un nom
de fichier contenant `|` est ambigu au réimport (suivi dans
[#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88)).

## Règles d'import

- **Validation stricte** : une IP invalide ou une date illisible **rejette
  la ligne avec un message précis** — jamais de dégradation silencieuse.
  L'import refusé ne mute pas le relevé en cours.
- **CSV** : séparateur virgule, en-têtes SFMS obligatoires.
- **XLSX** : première feuille lue avec le même schéma et la même validation
  que le CSV ; les colonnes supplémentaires (même insérées au milieu) sont
  ignorées tant que les en-têtes SFMS sont conservés ; les dates Excel sont
  normalisées au format SFMS.
- **Fusion** : l'import multi-fichiers fusionne les conversations
  identiques (compteurs additionnés, `origin` cumulé) ; la fusion
  inter-DLT est refusée.

## Protection tableur

Les cellules commençant par un caractère de formule (`=`, `+`, `-`, `@`)
sont préfixées par `'` à l'export (convention tableur, contre l'injection de
formule CSV). Le préfixe est retiré au réimport : l'aller-retour est
préservé.
