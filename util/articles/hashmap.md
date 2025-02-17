### Gestion des Doublons de Paquets Réseau avec des Tailles Différentes dans Sonar

Dans le développement de l’application Sonar, dédiée à la surveillance réseau,
un problème spécifique s'est posé : gérer efficacement les paquets réseau ayant
des tailles (`packet_size`) différentes sans les compter comme des paquets
distincts si tous les autres champs sont identiques. Voici un aperçu de ce
problème et de la solution que nous avons mise en place pour y remédier.

---

### Contexte

L'application Sonar enregistre des paquets réseau reçus sur différentes
interfaces et conserve des informations clés pour chaque paquet, notamment :

- Adresses MAC source et destination,
- Interface réseau,
- Protocole de la couche 3 (comme IPv4 ou IPv6),
- Informations des couches 3 et 4 (comme les adresses IP et les ports),
- Taille du paquet (`packet_size`).

Chaque paquet réseau est stocké dans une structure `PacketInfos`, et Sonar
conserve une matrice de trafic (`matrice`) qui suit le nombre d'occurrences de
chaque type de paquet en fonction des informations précitées.

### Problème

Lorsque deux paquets réseau sont identiques pour tous les champs sauf
`packet_size`, ils ne doivent pas être considérés comme des paquets différents
mais comme des variations du même paquet. Cependant, si la taille
(`packet_size`) est incluse dans la clé qui identifie chaque paquet unique,
chaque variation de taille génère un enregistrement distinct. Cela fausse les
statistiques de trafic en créant des doublons indésirables.

Ainsi, nous avons deux options pour gérer les doublons de `packet_size` :

1. **Ignorer la taille** dans l’identification unique des paquets et la
   remplacer par la dernière valeur reçue.
2. **Cumuler les valeurs de `packet_size` et `count`** dans un champ dédié de la
   structure de données, ce qui permet de suivre le nombre total de paquets et
   la taille totale cumulée de manière centralisée.

### Solution : Séparation des Clés et des Statistiques

Pour répondre à ces exigences, nous avons décidé de **séparer les clés
d’identification des paquets et les statistiques**. Voici les étapes de notre
solution :

1. **Créer une clé de paquet sans `packet_size`** : Nous avons introduit une
   nouvelle structure, `PacketKey`, qui contient toutes les informations
   pertinentes pour identifier un paquet unique sans tenir compte de sa taille.

2. **Ajouter une structure de statistiques `PacketStats`** : Cette structure
   gère le cumul de `count` (nombre de paquets similaires) et de
   `packet_size_total` (taille cumulée de tous les paquets ayant la même clé
   `PacketKey`).

3. **Remplacer `PacketInfos` par `PacketKey` comme clé de la matrice** : Cela
   permet d'éviter les doublons basés uniquement sur des variations de taille.

### Implémentation

La solution repose sur deux structures : `PacketKey` pour la clé sans
`packet_size` et `PacketStats` pour stocker le nombre d'occurrences et la taille
cumulée.

#### Structure `PacketKey` (Clé sans Taille)

```rust
#[derive(Debug, Serialize, Clone, Eq, PartialEq, Hash)]
struct PacketKey {
    mac_address_source: String,
    mac_address_destination: String,
    interface: String,
    l_3_protocol: String,
    layer_3_infos: Layer3Infos,
}
```

`PacketKey` contient toutes les informations nécessaires pour distinguer les
paquets sans inclure `packet_size`.

#### Structure `PacketStats` (Statistiques Cumulées)

```rust
#[derive(Debug, Serialize, Clone)]
struct PacketStats {
    count: u32,            // Nombre de paquets similaires
    packet_size_total: u32, // Taille totale cumulée des paquets
}

impl PacketStats {
    fn new(packet_size: u32) -> Self {
        Self {
            count: 1,
            packet_size_total: packet_size,
        }
    }

    fn update(&mut self, packet_size: u32) {
        self.count += 1;
        self.packet_size_total += packet_size;
    }
}
```

`PacketStats` permet de suivre deux éléments :

- `count` : le nombre de paquets similaires reçus,
- `packet_size_total` : la somme des tailles de tous les paquets ayant la même
  clé `PacketKey`.

#### Conversion de `PacketInfos` en `PacketKey`

Pour utiliser `PacketKey` comme clé de notre `HashMap`, nous ajoutons une
méthode de conversion simple :

```rust
impl From<&PacketInfos> for PacketKey {
    fn from(info: &PacketInfos) -> Self {
        PacketKey {
            mac_address_source: info.mac_address_source.clone(),
            mac_address_destination: info.mac_address_destination.clone(),
            interface: info.interface.clone(),
            l_3_protocol: info.l_3_protocol.clone(),
            layer_3_infos: info.layer_3_infos.clone(),
        }
    }
}
```

#### Mise à jour de la matrice avec les nouvelles structures

La méthode `update_matrice_with_packet` met à jour les statistiques en utilisant
`PacketKey` comme clé :

```rust
impl SonarState {
    pub fn update_matrice_with_packet(&mut self, new_packet: PacketInfos) {
        let packet_size = new_packet.packet_size;

        // Crée une clé sans `packet_size`
        let key = PacketKey::from(&new_packet);

        // Mise à jour ou insertion dans la matrice
        self.matrice
            .entry(key)
            .and_modify(|stats| stats.update(packet_size))
            .or_insert(PacketStats::new(packet_size));
    }
}
```

Dans cette méthode :

- Si le `PacketKey` existe déjà dans la matrice, nous appelons `update` pour
  incrémenter `count` et ajouter la taille du paquet à `packet_size_total`.
- Si le `PacketKey` est nouveau, un nouvel enregistrement `PacketStats` est
  ajouté avec un `count` initialisé à 1 et `packet_size_total` à la taille du
  premier paquet.

### Résultat

Cette approche résout efficacement le problème des doublons :

- Les paquets ayant des tailles différentes ne sont plus enregistrés plusieurs
  fois s'ils ont la même structure (`PacketKey`).
- La taille totale (`packet_size_total`) et le nombre d'occurrences (`count`) de
  chaque type de paquet sont calculés correctement.

### Conclusion

En séparant les clés d'identification des statistiques de paquets, nous avons pu
résoudre le problème des doublons de taille sans perdre d'information. Cette
solution permet à Sonar de traiter des données réseau complexes de manière
fiable et efficace.
