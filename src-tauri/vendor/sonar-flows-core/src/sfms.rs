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

/// Version du format de préambule écrite par cette version de SONAR. La
/// version 2 ajoute le contexte de relevé ([`SurveyContext`]) ; la lecture
/// des fichiers v1 est inchangée.
pub const SFMS_VERSION: &str = "2";

/// Marqueur de la ligne de préambule, en tête de fichier.
pub const PREAMBLE_MARKER: &str = "#SFMS";

/// Contexte d'un relevé : où et par quoi il a été capté (#154, tranche 2).
///
/// Ces informations n'existent pas dans le paquet : elles sont déclarées par
/// l'opérateur au moment de l'arrêt de la capture ou de l'enregistrement
/// (arbitrage du 04/08/2026, à la manière de Wireshark). Elles décrivent le
/// **relevé entier** et restent volontairement hors de la clé d'identité des
/// flux et des nœuds : le même paquet peut être vu par plusieurs capteurs, et
/// mettre le capteur dans la clé empêcherait de reconnaître ce recouvrement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurveyContext {
    /// Site d'audit (« Usine Nord », « Poste 63 kV »…).
    pub site: Option<String>,
    /// Capteur ayant réalisé le relevé (nom de la sonde ou de la machine).
    pub sensor: Option<String>,
    /// Interface d'écoute (`eth0`…), connue de SONAR en capture live.
    pub interface: Option<String>,
}

impl SurveyContext {
    /// Aucun champ renseigné : le préambule n'émet alors aucune clé de
    /// contexte (un export sans contexte reste identique à un export v1 à la
    /// version près).
    pub fn is_empty(&self) -> bool {
        self.site.is_none() && self.sensor.is_none() && self.interface.is_none()
    }

    /// Normalise une saisie utilisateur : une valeur vide ou uniquement
    /// blanche vaut « non renseigné », les blancs de bord sont retirés.
    pub fn normalized_field(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }
}

/// Métadonnées lues dans une ligne de préambule `#SFMS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfmsPreamble {
    /// Version du format annoncée par le fichier.
    pub version: String,
    /// Type de liaison du relevé (`dlt=`), absent si le fichier ne le
    /// déclare pas.
    pub link_type: Option<LinkType>,
    /// Contexte du relevé (v2). Vide pour un fichier v1.
    pub context: SurveyContext,
}

/// Encode une valeur libre pour tenir dans un champ `clé=valeur` séparé par
/// des espaces : tout ce qui n'est pas alphanumérique ASCII ou `-._~` passe
/// en `%XX` (UTF-8 octet par octet). Un nom de site avec espaces, accents ou
/// `=` traverse ainsi le préambule sans le rendre ambigu.
fn encode_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(*byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

/// Inverse de [`encode_value`]. Une séquence `%` invalide est une erreur :
/// une métadonnée illisible ne doit jamais être devinée.
fn decode_value(value: &str) -> std::result::Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = value
                .get(index + 1..index + 3)
                .ok_or_else(|| format!("échappement incomplet dans « {value} »"))?;
            decoded.push(
                u8::from_str_radix(hex, 16)
                    .map_err(|_| format!("échappement invalide « %{hex} » dans « {value} »"))?,
            );
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| format!("valeur non UTF-8 dans « {value} »"))
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

/// Ligne de préambule écrite en tête d'un export de matrice, sans contexte
/// de relevé (capture non qualifiée, exports internes).
pub fn format_preamble(link_type: Option<LinkType>) -> String {
    format_preamble_with_context(link_type, &SurveyContext::default())
}

/// Ligne de préambule portant le contexte du relevé (#154). Les champs non
/// renseignés n'émettent aucune clé : un export sans contexte reste
/// exactement l'ancienne ligne, à la version près.
pub fn format_preamble_with_context(
    link_type: Option<LinkType>,
    context: &SurveyContext,
) -> String {
    let mut line = format!("{PREAMBLE_MARKER} version={SFMS_VERSION}");
    if let Some(link_type) = link_type {
        line.push_str(&format!(" dlt={}", link_type_name(link_type)));
    }
    for (key, value) in [
        ("site", context.site.as_ref()),
        ("sensor", context.sensor.as_ref()),
        ("interface", context.interface.as_ref()),
    ] {
        if let Some(value) = value {
            line.push_str(&format!(" {key}={}", encode_value(value)));
        }
    }
    line
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
    let mut context = SurveyContext::default();
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
            "site" => context.site = Some(decode_value(value)?),
            "sensor" => context.sensor = Some(decode_value(value)?),
            "interface" => context.interface = Some(decode_value(value)?),
            // Clés inconnues tolérées : un fichier plus récent reste lisible.
            _ => {}
        }
    }

    Ok(Some(SfmsPreamble {
        version: version.unwrap_or_else(|| SFMS_VERSION.to_string()),
        link_type,
        context,
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

    /// Le contexte de relevé traverse le préambule sans perte, y compris des
    /// valeurs libres contenant espaces, accents, `=` et `%` — un nom de site
    /// est saisi par un humain, pas contraint (#154).
    #[test]
    fn survey_context_round_trips_free_text_values() {
        let context = SurveyContext {
            site: Some("Usine Nord — atelier 3".to_string()),
            sensor: Some("sonde=A 100%".to_string()),
            interface: Some("eth0".to_string()),
        };

        let line = format_preamble_with_context(Some(LinkType::ETHERNET), &context);
        assert!(
            !line.contains("Usine Nord"),
            "les espaces doivent être échappés : {line}"
        );

        let parsed = parse_preamble(&line)
            .expect("préambule écrit par nous")
            .expect("ligne de préambule");
        assert_eq!(parsed.context, context, "aller-retour exact");
        assert_eq!(parsed.link_type, Some(LinkType::ETHERNET));
        assert_eq!(parsed.version, "2");
    }

    /// Sans contexte, la ligne reste celle d'avant (à la version près) : un
    /// export non qualifié n'invente aucune métadonnée.
    #[test]
    fn a_preamble_without_context_emits_no_context_key() {
        let line = format_preamble(Some(LinkType::ETHERNET));
        assert_eq!(line, "#SFMS version=2 dlt=ETHERNET");

        let parsed = parse_preamble(&line).expect("valide").expect("préambule");
        assert!(parsed.context.is_empty());
    }

    /// Un relevé v1 se lit inchangé, avec un contexte simplement vide.
    #[test]
    fn a_v1_file_reads_with_an_empty_context() {
        let parsed = parse_preamble("#SFMS version=1 dlt=LINUX_SLL")
            .expect("valide")
            .expect("préambule");
        assert_eq!(parsed.version, "1");
        assert_eq!(parsed.link_type, Some(LinkType::LINUX_SLL));
        assert!(parsed.context.is_empty(), "aucun contexte inventé");
    }

    /// Un échappement corrompu est refusé plutôt que deviné.
    #[test]
    fn a_broken_escape_is_refused() {
        assert!(parse_preamble("#SFMS version=2 site=Usine%").is_err());
        assert!(parse_preamble("#SFMS version=2 site=Usine%ZZ").is_err());
    }

    /// Les saisies vides ou blanches valent « non renseigné » : le préambule
    /// ne se remplit pas de champs vides.
    #[test]
    fn blank_user_input_is_normalized_to_absent() {
        assert_eq!(SurveyContext::normalized_field(Some("   ")), None);
        assert_eq!(SurveyContext::normalized_field(Some("")), None);
        assert_eq!(SurveyContext::normalized_field(None), None);
        assert_eq!(
            SurveyContext::normalized_field(Some("  Poste 63 kV  ")),
            Some("Poste 63 kV".to_string())
        );
    }
}
