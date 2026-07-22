//! Validation du format des champs MAC/IP d'une ligne de labels, et
//! normalisation d'une clé `(mac, ip)` saisie dans l'UI de gestion.

use std::net::IpAddr;

use crate::errors::{CaptureStateError, label::LabelError};
use crate::setup::labels::clean_csv_field;

use super::csv::{LabelRow, is_header_row};

fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

fn is_mac_address(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Variante sur lignes déjà lues : évite de relire le fichier (#153).
pub(super) fn verif_mac_ip_format_rows(rows: &[LabelRow]) -> Result<(), CaptureStateError> {
    let mut invalid_ip: Vec<(usize, String, String)> = Vec::new();
    let mut invalid_mac: Vec<(usize, String, String)> = Vec::new();

    let skip = usize::from(rows.first().is_some_and(is_header_row));
    for row in rows.iter().skip(skip) {
        if !is_ip_address(&row.ip) && !row.ip.is_empty() {
            invalid_ip.push((row.line, row.ip.clone(), row.raw.clone()));
        }
        if !is_mac_address(&row.mac) && !row.mac.is_empty() {
            invalid_mac.push((row.line, row.mac.clone(), row.raw.clone()));
        }
    }

    if !invalid_ip.is_empty() || !invalid_mac.is_empty() {
        return Err(LabelError::InvalidMacIpFormat {
            invalid_mac,
            invalid_ip,
        }
        .into());
    }
    Ok(())
}

/// Normalise et valide une clé de label saisie dans l'UI de gestion (#157) :
/// MAC en minuscules, IP canonique (IPv6 compressée) — mêmes formes que le
/// parser de paquets et que l'import de fichier (#153). Un champ vide est
/// permis (label partiel), pas les deux. Erreurs au même format que l'import
/// (rendu identique côté front).
pub fn normalize_label_key(mac: &str, ip: &str) -> Result<(String, String), CaptureStateError> {
    let mac = clean_csv_field(mac).to_ascii_lowercase();
    let ip_raw = clean_csv_field(ip);
    let raw = format!("{mac},{ip_raw}");

    let mut invalid_mac = Vec::new();
    let mut invalid_ip = Vec::new();
    if !mac.is_empty() && !is_mac_address(&mac) {
        invalid_mac.push((1, mac.clone(), raw.clone()));
    }
    let ip = if ip_raw.is_empty() {
        String::new()
    } else {
        match ip_raw.parse::<IpAddr>() {
            Ok(addr) => addr.to_string(),
            Err(_) => {
                invalid_ip.push((1, ip_raw.to_string(), raw.clone()));
                ip_raw.to_string()
            }
        }
    };
    if !invalid_mac.is_empty() || !invalid_ip.is_empty() {
        return Err(LabelError::InvalidMacIpFormat {
            invalid_mac,
            invalid_ip,
        }
        .into());
    }
    if mac.is_empty() && ip.is_empty() {
        return Err(LabelError::InvalidRowsFormat {
            invalid_lines: vec![(1, "MAC et IP tous deux vides".to_string())],
        }
        .into());
    }
    Ok((mac, ip))
}

// La production passe par `verif_mac_ip_format_rows` (lecture unique du
// fichier dans `import_label_file`) ; cette version basée fichier ne sert
// plus qu'aux tests.
#[cfg(test)]
pub(super) fn verif_mac_ip_format(csv_path: String) -> Result<(), CaptureStateError> {
    let rows = super::csv::read_label_rows(&csv_path)?;
    verif_mac_ip_format_rows(&rows)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_support::TempDir;
    use super::*;
    use std::fs;

    /// La saisie du panneau de gestion suit les mêmes règles que l'import de
    /// fichier : normalisation vers les formes du parser, rejet explicite.
    #[test]
    fn normalize_label_key_normalizes_and_rejects() {
        // MAC en minuscules, IPv6 compressée.
        let (mac, ip) = normalize_label_key("AA:BB:CC:DD:EE:FF", "2001:0db8:0::0001").unwrap();
        assert_eq!(mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(ip, "2001:db8::1");

        // Champ vide autorisé (label partiel), pas les deux.
        let (mac, ip) = normalize_label_key("", "192.168.1.10").unwrap();
        assert_eq!((mac.as_str(), ip.as_str()), ("", "192.168.1.10"));
        assert!(normalize_label_key("", "").is_err());

        // Valeurs invalides rejetées.
        assert!(normalize_label_key("pas-une-mac", "192.168.1.10").is_err());
        assert!(normalize_label_key("aa:bb:cc:dd:ee:ff", "999.1.1.1").is_err());
    }

    #[test]
    fn valid_mac_and_ip_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn malformed_ip_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "mac,ip,label\naa:bb:cc:dd:ee:ff,192.168.11,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::InvalidMacIpFormat {
                invalid_mac,
                invalid_ip,
            }) => {
                assert!(invalid_mac.is_empty());
                assert_eq!(
                    invalid_ip,
                    vec![(
                        2,
                        "192.168.11".to_string(),
                        "aa:bb:cc:dd:ee:ff,192.168.11,mon-pc".to_string()
                    )]
                );
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn malformed_mac_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:e:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_err());
    }

    #[test]
    fn empty_mac_and_ip_are_accepted() {
        let dir = TempDir::new("sonar_test_empty_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,,mon-pc\n,192.168.1.1,mon-pc\n,,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn empty_file_mac_ip_format_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }
}
