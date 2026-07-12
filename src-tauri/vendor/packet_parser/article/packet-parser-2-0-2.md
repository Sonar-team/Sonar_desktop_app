# Packet Parser 2.0.2: moins d'allocations, plus de types, plus de tests

`packet_parser` est une crate Rust pour analyser des paquets reseau couche par couche: Ethernet, IP, transport, puis protocoles applicatifs quand le payload peut etre reconnu.

Les dernieres modifications autour des versions 2.0.0, 2.0.1 et 2.0.2 ne changent pas seulement quelques details internes. Elles vont dans une direction assez nette: rendre la crate plus proche des donnees binaires qu'elle manipule, reduire les allocations inutiles, mesurer les performances entre versions, et renforcer les tests des parseurs applicatifs.

```toml
[dependencies]
packet_parser = "2.0.2"
```

Depot: https://github.com/Akmot9/Packet-parser

## Pourquoi cette serie de changements compte

Un parseur reseau vit dans une zone delicate.

Il doit etre rapide, parce qu'il peut etre appele sur beaucoup de paquets. Il doit etre strict, parce qu'un payload malforme ne doit pas etre accepte trop facilement. Il doit aussi rester exploitable par du code utilisateur: logs, JSON, dashboards, detection de protocoles, tests de non-regression.

Les versions recentes de `packet_parser` travaillent sur ces trois axes:

- une representation interne plus typee pour la couche Ethernet;
- des noms de protocoles sous forme de references statiques plutot que de `String`;
- un outil de benchmark multi-versions;
- une couverture de tests plus large pour les parseurs et les heuristiques de detection.

Ce sont des changements moins visibles qu'un nouveau protocole spectaculaire, mais ils rendent la crate plus saine pour les usages reels.

## DataLink stocke maintenant des donnees binaires

Le changement le plus important de la serie 2.0.0 concerne la couche `DataLink`.

Avant, certains champs etaient pratiques pour l'affichage, mais moins naturels pour du parsing: une adresse MAC ou un EtherType finissaient facilement sous forme de chaine. C'est lisible, mais ce n'est pas la forme native de ces donnees.

La structure `DataLink` utilise maintenant des types dedies:

```rust
pub struct DataLink<'a> {
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    pub vlan: Option<VlanTag>,
    pub ethertype: Ethertype,
    pub payload: &'a [u8],
}
```

`MacAddress` encapsule les six octets de l'adresse MAC:

```rust
pub struct MacAddress(pub [u8; 6]);
```

`Ethertype` encapsule le code `u16`:

```rust
pub struct Ethertype(pub u16);
```

Ce changement est plus coherent avec le domaine reseau. Une adresse MAC est une valeur de 48 bits. Un EtherType est une valeur de 16 bits. Les stocker directement sous forme binaire evite de transformer trop tot les donnees en texte.

Pour le code utilisateur, cela veut dire que les comparaisons peuvent devenir plus explicites:

```rust
use packet_parser::parse::data_link::ethertype::Ethertype;
use packet_parser::parse::data_link::DataLink;

fn inspect_ethernet(frame: &[u8]) {
    let datalink = DataLink::try_from(frame).expect("valid Ethernet frame");

    if datalink.ethertype == Ethertype::from(0x0800) {
        println!("IPv4 frame");
    }
}
```

La crate garde quand meme une sortie lisible quand c'est utile. `Ethertype::name()` permet d'obtenir un nom humain, et la serialization garde des valeurs faciles a lire:

```rust
println!("{}", datalink.ethertype.name());
println!("{}", datalink.destination_mac);
```

L'objectif n'est donc pas de rendre l'API moins pratique. L'objectif est de separer proprement deux besoins:

- la representation interne, qui doit rester precise et peu couteuse;
- la presentation externe, qui peut etre convertie en texte au moment ou elle est vraiment necessaire.

## La compatibilite JSON reste lisible

Un risque classique quand on rend une API plus typee est de casser les usages autour de la serialization.

Ici, `MacAddress` se serialise toujours sous forme de chaine hexadecimale:

```json
"2c:fd:a1:3c:4d:5e"
```

Et `Ethertype` se serialise sous forme de nom quand il est connu:

```json
"IPv4"
```

Pour un EtherType inconnu, la crate garde une forme lisible:

```json
"Unknown (0xFFFF)"
```

C'est un compromis utile: le code Rust peut manipuler des types binaires, pendant que les exports JSON restent comprehensibles pour les dashboards, les logs et les outils d'analyse.

## Moins d'allocations pour les noms de protocoles

Un autre changement recent concerne les noms de protocoles.

Plusieurs structures utilisaient des `String` pour porter des noms comme `IPv4`, `ARP`, `DNS`, `TLS` ou `PostgreSQL`. Ces noms sont connus a la compilation. Il n'est pas necessaire d'allouer une nouvelle chaine a chaque paquet parse.

La crate utilise maintenant des references statiques dans les chemins de parsing:

```rust
pub struct Application {
    pub application_protocol: &'static str,
}
```

Le meme principe est applique aux noms de protocoles des autres couches quand c'est possible.

Ce genre de changement parait petit, mais il compte dans une bibliotheque appelee en boucle sur des captures reseau. Chaque allocation evitee rend le chemin de parsing plus previsible.

Il y a aussi un benefice de lisibilite: un protocole reconnu par le parseur est exprime comme une constante logique, pas comme une chaine construite dynamiquement.

## Des benchmarks entre versions avec verbench

La version 2.0.1 ajoute `verbench`, un outil de benchmark multi-versions place dans `tools/verbench`.

Son role est simple: mesurer les timings de parsing par couche OSI pour plusieurs versions publiees de la crate, puis comparer ces resultats avec la copie locale.

Le fichier produit ressemble a ceci:

```json
{
  "1.5.5": {
    "l2_ns": 567,
    "l3_ns": 957,
    "l4_ns": 49,
    "l7_ns": 196,
    "total_ns": 1944
  },
  "2.0.0-local": {
    "l2_ns": 38,
    "l3_ns": 58,
    "l4_ns": 49,
    "l7_ns": 137,
    "total_ns": 454
  }
}
```

Ces chiffres doivent etre lus avec prudence: ils dependent de la machine, du paquet de reference, du compilateur, et des conditions d'execution. Mais l'outil donne une chose importante au projet: un moyen reproductible de voir si une modification accelere ou ralentit le parsing.

Pour une crate de parsing, c'est beaucoup plus utile qu'une impression generale.

La commande principale est:

```bash
tools/verbench/run.sh
```

Elle regenere `perf_by_version.json` a la racine du depot.

## Une detection applicative plus verifiee

La version 2.0.2 ajoute surtout de la couverture de tests.

Les tests ne se limitent pas a verifier que les parseurs acceptent quelques paquets valides. Ils verifient aussi que les detecteurs ne classent pas trop vite un payload comme un protocole connu.

C'est un point important dans `packet_parser`, parce que plusieurs detections applicatives ne doivent pas dependre uniquement du port. Par exemple, un payload PostgreSQL peut etre reconnu par sa structure, pas seulement parce qu'il circule sur le port 5432.

La detection applicative couvre notamment:

- NTP;
- Bitcoin;
- OPC UA;
- EtherNet/IP;
- PostgreSQL;
- DNS;
- SNMP;
- TLS;
- S7Comm;
- GIOP;
- SRVLOC;
- Modbus/TCP;
- QUIC.

La logique de detection doit rester stricte. Un parseur trop permissif creerait des faux positifs, surtout sur des captures TCP ou UDP heterogenes.

## PostgreSQL: detection structurelle et faux positifs

Le parseur PostgreSQL a recu une attention particuliere.

La crate sait deja parser plusieurs formes du protocole:

- `StartupMessage`;
- `SSLRequest`;
- `GSSENCRequest`;
- `CancelRequest`;
- messages types comme `Query`, `Bind`, `Parse`, `ReadyForQuery`, `ErrorResponse`, `NoticeResponse`, etc.

Les tests recents renforcent surtout l'heuristique `is_likely_postgresql_payload`.

L'enjeu est de reconnaitre un vrai payload PostgreSQL sans accepter des sequences trop faibles. Par exemple, un unique message `ReadyForQuery` ou `BackendKeyData` peut etre syntaxiquement valide, mais pas forcement suffisant pour conclure qu'un flux arbitraire est du PostgreSQL.

Le parseur cherche donc des preuves plus solides:

- une sequence de demarrage coherente;
- des messages SQL plausibles;
- des corps d'erreur ou de notice structures;
- des longueurs correctes;
- des chaines terminees proprement;
- des codes d'authentification compatibles.

C'est exactement le genre de rigueur necessaire quand la detection ne repose pas seulement sur le port.

## SNMP: plus de cas couverts

SNMP est un autre bon exemple.

Le parseur manipule des structures ASN.1/BER, avec des formes differentes selon les versions SNMP v1, v2c et v3. Les tests recents couvrent davantage de cas:

- trap SNMPv1;
- adresses IP d'agent invalides;
- valeurs SNMP comme `Integer`, `OctetString`, `ObjectIdentifier`, `Counter32`, `Gauge32`, `TimeTicks`, `Counter64`;
- exceptions comme `NoSuchObject`, `NoSuchInstance` et `EndOfMibView`;
- PDU scopee en SNMPv3;
- PDU chiffre en SNMPv3;
- longueurs BER en forme longue;
- rejet des longueurs indefinies ou trop grandes.

La valeur de ces tests est double.

D'abord, ils documentent le comportement attendu du parseur. Ensuite, ils protegent les futurs changements contre des regressions discretes, typiques des parseurs binaires.

## MQTT et les parseurs de protocole

La couverture MQTT a aussi ete etendue.

Les tests verifient plusieurs formes de paquets:

- `CONNECT`;
- `CONNACK`;
- `PUBLISH`;
- `PINGREQ`;
- `SUBSCRIBE`;
- erreurs de type de paquet;
- flags invalides;
- longueur restante encodee incorrectement;
- topic length invalide.

Cela renforce le parseur lui-meme, meme si la detection applicative globale ne doit pas necessairement activer tous les protocoles dans tous les contextes.

La difference est importante: un protocole peut avoir un parseur robuste, et la strategie de detection peut rester volontairement conservatrice.

## Ce que cela change pour les utilisateurs

Si vous utilisez `packet_parser` comme bibliotheque de parsing, les changements les plus visibles sont probablement autour de `DataLink`.

Au lieu de traiter les MAC et les EtherTypes comme du texte, il vaut mieux les manipuler comme des valeurs:

```rust
use packet_parser::parse::data_link::ethertype::Ethertype;
use packet_parser::PacketFlow;

fn inspect_frame(frame: &[u8]) {
    let flow = PacketFlow::try_from(frame).expect("valid packet flow");

    if flow.data_link.ethertype == Ethertype::from(0x86DD) {
        println!("IPv6");
    }

    println!("source mac: {}", flow.data_link.source_mac);
    println!("ethertype: {}", flow.data_link.ethertype.name());
}
```

Pour un export ou une vue utilisateur, il reste possible d'obtenir des chaines. Pour de la logique metier, les types binaires sont plus robustes.

## Une crate plus mature

Cette serie de modifications donne une impression assez claire: `packet_parser` passe d'une crate qui accumule des parseurs a une crate qui consolide ses fondations.

La representation des donnees est plus proche du reseau. Les allocations inutiles reculent. Les performances deviennent mesurables version par version. Les parseurs applicatifs sont testes plus en profondeur, avec une attention particuliere aux faux positifs.

C'est une direction saine pour un projet de parsing reseau.

Dans ce domaine, la qualite ne se voit pas seulement au nombre de protocoles supportes. Elle se voit aussi dans les cas rejetes, dans les erreurs precises, dans la stabilite des exports, et dans la capacite a faire evoluer le code sans casser silencieusement les hypotheses de detection.

Avec la serie 2.0.x, `packet_parser` avance clairement dans ce sens.
