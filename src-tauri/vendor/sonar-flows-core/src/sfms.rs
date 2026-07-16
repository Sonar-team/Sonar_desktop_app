//! Préambule SFMS : métadonnées du relevé portées par la première ligne des
//! exports CSV (arbitrage du 14/07/2026).
//!
//! Un relevé = un réseau = un type de liaison : plutôt qu'une colonne DLT
//! répétée à chaque ligne, l'export écrit une ligne `#SFMS key=value …` avant
//! la ligne d'en-têtes. Un export antérieur à ce préambule s'importe avec un
//! DLT implicite Ethernet. Le préambule ne porte volontairement **pas** de
//! date d'export : deux exports de la même matrice doivent rester identiques
//! octet pour octet (déterminisme, #148).

use std::io::BufRead;
use std::path::Path;

use packet_parser::LinkType;

use crate::{Result, SonarCoreError};

/// Version du format de préambule écrite par cette version de SONAR.
pub const SFMS_VERSION: &str = "1";

/// Marqueur de la ligne de préambule, en tête de fichier.
pub const PREAMBLE_MARKER: &str = "#SFMS";

/// Métadonnées lues dans une ligne de préambule `#SFMS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfmsPreamble {
    /// Version du format annoncée par le fichier.
    pub version: String,
    /// Type de liaison du relevé (`dlt=`), absent si le fichier ne le
    /// déclare pas.
    pub link_type: Option<LinkType>,
}

/// Nom canonique d'un type de liaison pour le préambule et les messages
/// d'erreur : mnémonique `LINKTYPE_*` pour les liaisons décodées par
/// `packet_parser`, valeur numérique sinon — jamais de nom inventé.
pub fn link_type_name(link_type: LinkType) -> String {
    match link_type {
        LinkType::ETHERNET => "ETHERNET".to_string(),
        LinkType::RAW => "RAW".to_string(),
        LinkType::IEEE802_11 => "IEEE802_11".to_string(),
        LinkType::LINUX_SLL => "LINUX_SLL".to_string(),
        LinkType::LINUX_SLL2 => "LINUX_SLL2".to_string(),
        other => other.0.to_string(),
    }
}

/// Inverse exact de [`link_type_name`] : mnémonique connu ou valeur
/// numérique. `None` si le texte n'est ni l'un ni l'autre.
pub fn link_type_from_text(text: &str) -> Option<LinkType> {
    match text {
        "ETHERNET" => Some(LinkType::ETHERNET),
        "RAW" => Some(LinkType::RAW),
        "IEEE802_11" => Some(LinkType::IEEE802_11),
        "LINUX_SLL" => Some(LinkType::LINUX_SLL),
        "LINUX_SLL2" => Some(LinkType::LINUX_SLL2),
        other => other.parse::<u32>().ok().map(LinkType),
    }
}

/// Ligne de préambule écrite en tête d'un export de matrice.
pub fn format_preamble(link_type: Option<LinkType>) -> String {
    match link_type {
        Some(link_type) => format!(
            "{PREAMBLE_MARKER} version={SFMS_VERSION} dlt={}",
            link_type_name(link_type)
        ),
        None => format!("{PREAMBLE_MARKER} version={SFMS_VERSION}"),
    }
}

/// Parse une ligne de préambule. `Ok(None)` si la ligne n'en est pas une
/// (fichier d'un export antérieur) ; erreur si le préambule est présent mais
/// illisible — une métadonnée qu'on ne sait pas lire ne doit pas être
/// silencieusement ignorée. Les clés inconnues sont tolérées (extensions
/// futures du format).
pub fn parse_preamble(line: &str) -> std::result::Result<Option<SfmsPreamble>, String> {
    let Some(fields) = line.trim_end().strip_prefix(PREAMBLE_MARKER) else {
        return Ok(None);
    };

    let mut version: Option<String> = None;
    let mut link_type: Option<LinkType> = None;
    for field in fields.split_whitespace() {
        let Some((key, value)) = field.split_once('=') else {
            return Err(format!("champ de préambule invalide : « {field} »"));
        };
        match key {
            "version" => version = Some(value.to_string()),
            "dlt" => {
                link_type = Some(link_type_from_text(value).ok_or_else(|| {
                    format!("type de liaison illisible dans le préambule : « {value} »")
                })?);
            }
            // Clés inconnues tolérées : un fichier plus récent reste lisible.
            _ => {}
        }
    }

    Ok(Some(SfmsPreamble {
        version: version.unwrap_or_else(|| SFMS_VERSION.to_string()),
        link_type,
    }))
}

/// Lit le préambule d'un fichier de matrice CSV (sa première ligne).
/// `Ok(None)` pour un export antérieur au préambule.
pub fn read_preamble(path: &Path) -> Result<Option<SfmsPreamble>> {
    let file = std::fs::File::open(path).map_err(|e| SonarCoreError::InvalidCsv {
        path: path.to_path_buf(),
        message: format!("ouverture impossible: {e}"),
    })?;
    let mut first_line = String::new();
    std::io::BufReader::new(file)
        .read_line(&mut first_line)
        .map_err(|e| SonarCoreError::InvalidCsv {
            path: path.to_path_buf(),
            message: format!("lecture impossible: {e}"),
        })?;
    parse_preamble(&first_line).map_err(|message| SonarCoreError::InvalidCsv {
        path: path.to_path_buf(),
        message,
    })
}

/// Type de liaison déclaré par un fichier de matrice : celui du préambule,
/// sinon Ethernet implicite (exports antérieurs au préambule, arbitrage du
/// 14/07/2026).
pub fn matrix_file_link_type(path: &Path) -> Result<LinkType> {
    Ok(read_preamble(path)?
        .and_then(|preamble| preamble.link_type)
        .unwrap_or(LinkType::ETHERNET))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preamble_round_trips_every_supported_link_type() {
        for link_type in [
            LinkType::ETHERNET,
            LinkType::RAW,
            LinkType::IEEE802_11,
            LinkType::LINUX_SLL,
            LinkType::LINUX_SLL2,
            LinkType(147), // DLT_USER0 : pas de mnémonique, valeur numérique
        ] {
            let line = format_preamble(Some(link_type));
            let parsed = parse_preamble(&line)
                .expect("préambule écrit par nous")
                .expect("ligne de préambule");
            assert_eq!(parsed.link_type, Some(link_type), "aller-retour {line}");
            assert_eq!(parsed.version, SFMS_VERSION);
        }
    }

    #[test]
    fn a_regular_header_line_is_not_a_preamble() {
        assert_eq!(parse_preamble("mac_source,mac_destination,…"), Ok(None));
    }

    #[test]
    fn unknown_keys_are_tolerated_but_bad_dlt_is_an_error() {
        let parsed = parse_preamble("#SFMS version=1 dlt=ETHERNET first_seen=2026-07-14")
            .expect("clé inconnue tolérée")
            .expect("préambule");
        assert_eq!(parsed.link_type, Some(LinkType::ETHERNET));

        assert!(parse_preamble("#SFMS version=1 dlt=N_IMPORTE_QUOI").is_err());
        assert!(parse_preamble("#SFMS champ_sans_valeur").is_err());
    }
}
