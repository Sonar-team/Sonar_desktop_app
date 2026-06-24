# Methode de travail pour ajouter un nouveau protocole

Ce document decrit la methode a suivre pour ajouter un protocole dans la librairie. Le but est de garder le meme style que le code existant : parsing par couche, erreurs typpees, validations separees, et integration explicite dans les modules.

## Principes obligatoires

- Le parseur du protocole doit implementer `TryFrom<&[u8]>`.
- Le parseur ne doit pas consommer les donnees : il travaille sur une slice et garde des references avec des lifetimes quand c'est utile.
- La strategie par defaut est zero-copy : aucune copie du paquet, du payload ou des champs variables.
- Les erreurs doivent utiliser `thiserror`.
- Les erreurs doivent etre dans un fichier dedie au protocole.
- Les validations et extractions controlees doivent etre dans un fichier separe du parseur.
- Le parseur doit retourner une erreur specifique des qu'une valeur de structure est invalide.
- La rustdoc du type principal doit documenter le format du paquet avec un schema Mermaid `packet-beta`.
- Les tests doivent couvrir au minimum un paquet valide, les tailles invalides, et les champs invalides importants.
- Chaque fichier source ajoute pour un protocole doit contenir l'en-tete de licence MIT du projet.
- Les constantes doivent etre nommees en majuscules, avec des underscores entre les mots si necessaire.

## Strategie zero-copy et usage temps reel

La librairie doit pouvoir s'adapter a des usages temps reel, par exemple des moteurs de capture reseau. Le parsing doit donc eviter les allocations et les copies inutiles. Le paquet original reste la source de verite, et les structures parsees doivent pointer vers des zones du paquet quand les donnees sont variables.

Regles a suivre :

- Utiliser `&'a [u8]` pour les payloads, options, donnees brutes et champs de taille variable.
- Utiliser des types scalaires (`u8`, `u16`, `u32`, `u64`, bool) pour les champs fixes.
- Utiliser `from_be_bytes`, `from_le_bytes` ou des operations bitwise pour extraire les champs numeriques.
- Valider les tailles avant chaque indexation ou slicing.
- Eviter `Vec<u8>`, `String`, `to_vec()`, `to_string()` et `clone()` dans les structures de parsing.
- Eviter de reconstruire un paquet ou une sous-partie du paquet.
- Representer les champs texte en `&'a str` seulement si le protocole impose du texte UTF-8 et que la validation est faite ; sinon garder `&'a [u8]`.
- Les conversions couteuses doivent etre placees dans des helpers optionnels ou dans la couche d'affichage, pas dans le parsing de base.

`from_be_bytes` reste autorise et recommande pour les champs numeriques reseau. Il copie seulement quelques octets dans une valeur scalaire, par exemple 2 octets pour un `u16` ou 4 octets pour un `u32`. Ce n'est pas une allocation et ce n'est pas une copie du paquet.

Exemple correct :

```rust
pub struct FooPacket<'a> {
    pub message_type: u8,
    pub length: u16,
    pub payload: &'a [u8],
}

let length = u16::from_be_bytes([packet[2], packet[3]]);
let payload = &packet[4..length as usize];
```

Exemple a eviter dans un parseur zero-copy :

```rust
pub struct FooPacket {
    pub payload: Vec<u8>,
    pub label: String,
}

let payload = packet[4..].to_vec();
let label = String::from_utf8_lossy(&packet[0..4]).to_string();
```

Si un nom de protocole ou une valeur lisible doit etre exposee, preferer un enum ou une reference statique :

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooMessageType {
    Request,
    Response,
    Unknown(u8),
}

pub struct Application {
    pub application_protocol: &'static str,
}
```

Les allocations restent possibles uniquement si elles sont justifiees par le protocole ou par une API de plus haut niveau. Dans ce cas, le choix doit etre explicite et ne pas etre place dans le chemin critique du parsing temps reel.

## Organisation des fichiers

La librairie est organisee par couche :

```text
src/
  parse/
    data_link/
    internet/
    transport/
    application/
  errors/
    data_link/
    internet/
    transport/
    application/
  checks/
    data_link/
    transport/
    application/
```

Pour un nouveau protocole, ajouter les fichiers dans la couche concernee.

Exemple pour un protocole applicatif `foo` :

```text
src/parse/application/protocols/foo.rs
src/errors/application/foo.rs
src/checks/application/foo.rs
```

Si le protocole devient trop gros et necessite plusieurs sous-modules, utiliser un dossier :

```text
src/parse/application/protocols/foo/mod.rs
src/parse/application/protocols/foo/...
```

Dans ce cas, garder la meme separation logique. Le fichier d'erreur peut s'appeler `error.rs` dans le dossier du protocole seulement si le protocole est autonome en sous-modules, mais la convention actuelle du projet est de centraliser les erreurs dans `src/errors/<couche>/<protocole>.rs`.

## 1. Creer le type d'erreur

Les erreurs doivent etre precises et exploitables. Ne pas utiliser `String` comme erreur principale si une variante dediee est possible.

Exemple :

```rust
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum FooError {
    #[error("Packet too short: expected at least {expected} bytes, got {actual} bytes")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid Foo version: {0}")]
    InvalidVersion(u8),

    #[error("Invalid Foo message type: {0}")]
    InvalidMessageType(u8),
}
```

Puis exporter le module dans le `mod.rs` de la couche :

```rust
pub mod foo;
```

Exemple existant a suivre : `src/errors/transport/tcp.rs` et `src/errors/application/ntp.rs`.

## 2. Creer les validations dans `checks`

Les controles de taille, de bornes et de coherence doivent etre sortis du parseur quand ils ne sont pas triviaux. Le parseur doit appeler ces fonctions avant de lire les octets concernes.

Exemple :

```rust
use crate::errors::application::foo::FooError;

const FOO_MIN_LENGTH: usize = 8;

pub fn validate_foo_min_length(packet: &[u8]) -> Result<(), FooError> {
    if packet.len() < FOO_MIN_LENGTH {
        return Err(FooError::InvalidLength {
            expected: FOO_MIN_LENGTH,
            actual: packet.len(),
        });
    }
    Ok(())
}

pub fn validate_foo_version(version: u8) -> Result<(), FooError> {
    if version != 1 {
        return Err(FooError::InvalidVersion(version));
    }
    Ok(())
}
```

Puis exporter le module dans le `mod.rs` de la couche :

```rust
pub mod foo;
```

Exemples existants a suivre : `src/checks/transport/tcp/mod.rs` et `src/checks/application/ntp.rs`.

## 3. Implementer le parseur avec `TryFrom<&[u8]>`

Le fichier de parsing doit seulement :

1. appeler les validations ;
2. extraire les champs avec `from_be_bytes` ou des operations de bits claires ;
3. construire la structure ;
4. separer le header et le payload si le protocole en possede un.

Exemple :

```rust
use std::convert::TryFrom;

use crate::{
    checks::application::foo::{validate_foo_min_length, validate_foo_version},
    errors::application::foo::FooError,
};

#[derive(Debug, PartialEq)]
pub struct FooPacket<'a> {
    pub version: u8,
    pub message_type: u8,
    pub length: u16,
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for FooPacket<'a> {
    type Error = FooError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        validate_foo_min_length(packet)?;

        let version = packet[0] >> 4;
        validate_foo_version(version)?;

        let message_type = packet[1];
        let length = u16::from_be_bytes([packet[2], packet[3]]);

        if packet.len() < length as usize {
            return Err(FooError::InvalidLength {
                expected: length as usize,
                actual: packet.len(),
            });
        }

        Ok(FooPacket {
            version,
            message_type,
            length,
            payload: &packet[4..length as usize],
        })
    }
}
```

Points importants :

- Toujours valider la longueur avant d'indexer dans la slice.
- Ne pas faire de `unwrap()` dans le parseur.
- Utiliser `u16::from_be_bytes`, `u32::from_be_bytes`, etc. pour les valeurs reseau.
- Garder les slices empruntees (`&'a [u8]`) pour eviter les allocations inutiles.
- Les valeurs reservees, versions, flags, longueurs et types doivent etre verifies.

Exemples existants a suivre : `src/parse/transport/protocols/tcp.rs`, `src/parse/internet/protocols/ipv4.rs`, `src/parse/application/protocols/ntp.rs`.

## 4. Ajouter la rustdoc avec schema Mermaid

Le type principal du protocole doit avoir une rustdoc qui decrit le format du paquet. Quand le format est fixe ou partiellement fixe, ajouter un schema Mermaid `packet-beta`, comme dans `src/parse/application/protocols/s7comm.rs`.

Le schema doit servir a comprendre rapidement les offsets, tailles et champs importants avant de lire le code.

Exemple :

````rust
#[cfg_attr(doc, aquamarine::aquamarine)]
/// Foo Protocol Packet
///
/// ```mermaid
/// ---
/// title: FooPacket
/// ---
/// packet-beta
/// 0-3: "Version u4"
/// 4-7: "Flags u4"
/// 8-15: "Message Type u8"
/// 16-31: "Length u16"
/// 32-63: "Transaction ID u32"
/// 64-95: "Checksum u32"
/// 96-127: "Payload / Options"
/// ```
#[derive(Debug, PartialEq)]
pub struct FooPacket<'a> {
    pub version: u8,
    pub flags: u8,
    pub message_type: u8,
    pub length: u16,
    pub transaction_id: u32,
    pub checksum: u32,
    pub payload: &'a [u8],
}
````

Regles pour le schema :

- Utiliser `packet-beta`.
- Indiquer les plages de bits ou d'octets de facon coherente avec le protocole.
- Nommer les champs avec leur taille (`u8`, `u16`, `u32`, `slice`, etc.) quand c'est utile.
- Documenter les champs variables avec une zone explicite, par exemple `"Payload variable"` ou `"Options variable"`.
- Garder le schema proche de la structure parsee.
- Ne pas remplacer les validations par la documentation : la rustdoc explique, le code valide.

## 5. Integrer le protocole dans les modules

Ajouter le module dans le `mod.rs` de la couche concernee.

Exemple application :

```rust
pub mod foo;
```

Ajouter le type dans l'enum si la couche expose une enum de protocoles.

Exemple dans `src/parse/application/protocols/mod.rs` :

```rust
use foo::FooPacket;

pub mod foo;

#[derive(Debug)]
pub enum ApplicationProtocol<'a> {
    Foo(FooPacket<'a>),
    Raw(&'a [u8]),
    None,
}
```

Si le protocole doit etre detecte automatiquement, l'ajouter dans le `TryFrom<&[u8]>` de la couche concernee.

Exemple dans `src/parse/application/mod.rs` :

```rust
if FooPacket::try_from(packet).is_ok() {
    return Ok(Application {
        application_protocol: "Foo".to_string(),
    });
}
```

Attention : l'ordre de detection compte. Un parseur trop permissif peut capturer des paquets qui appartiennent a un autre protocole. Les validations doivent donc etre assez strictes avant d'ajouter le protocole dans la detection automatique.

## 6. Ajouter les tests

Chaque protocole doit avoir des tests unitaires proches du parseur ou des checks.

Tests minimum :

- paquet valide ;
- paquet trop court ;
- version invalide ;
- longueur annoncee invalide ;
- type ou flag invalide ;
- payload vide si c'est autorise ;
- payload present si le protocole en possede un.

Exemple :

```rust
#[test]
fn test_foo_packet_too_short() {
    let packet = [0u8; 3];
    let result = FooPacket::try_from(&packet[..]);

    assert!(matches!(
        result,
        Err(FooError::InvalidLength { expected: 8, actual: 3 })
    ));
}
```

Les erreurs doivent etre testees avec `matches!` quand elles contiennent des champs.

## 7. Checklist avant commit

- Le protocole implemente `TryFrom<&[u8]>`.
- Les erreurs sont dans un fichier dedie et utilisent `thiserror::Error`.
- Les validations sont dans un fichier separe sous `src/checks`.
- Le type principal a une rustdoc avec un schema Mermaid `packet-beta`.
- Les `mod.rs` necessaires exportent le nouveau module.
- Les enums de protocoles sont mises a jour si necessaire.
- La detection automatique est ajoutee seulement si les validations sont strictes.
- Les tests couvrent les cas valides et invalides.
- `cargo fmt` passe.
- `cargo test` passe.

## Commandes utiles

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```
