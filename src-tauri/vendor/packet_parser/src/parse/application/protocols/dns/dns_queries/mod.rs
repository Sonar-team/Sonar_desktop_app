use std::fmt;

use crate::{
    errors::application::dns::DnsQueryParseError,
    parse::application::protocols::dns::utils::{dns_class::DnsClass, dns_types::DnsType},
};

#[derive(Debug, PartialEq)]
pub struct DnsQueries {
    pub queries: Vec<DnsQuery>,
}

impl DnsQueries {
    pub fn from_bytes(bytes: &[u8], count: u16) -> Result<Self, DnsQueryParseError> {
        let mut queries = Vec::with_capacity(count as usize);
        let mut offset = 0;
        for _ in 0..count {
            check_dns_query_size(bytes, offset, 1)?;
            queries.push(DnsQuery::from_bytes(bytes, &mut offset)?);
        }
        Ok(DnsQueries { queries })
    }
}

fn check_dns_query_size(
    bytes: &[u8],
    offset: usize,
    required_size: usize,
) -> Result<(), DnsQueryParseError> {
    if offset + required_size > bytes.len() {
        return Err(DnsQueryParseError::InsufficientData {
            required: required_size,
            offset,
            available: bytes.len() - offset,
        });
    }
    Ok(())
}

impl fmt::Display for DnsQueries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DnsQueries {{ queries: [")?;
        for query in &self.queries {
            write!(f, " {query},")?;
        }
        write!(f, "] }}")
    }
}

#[derive(Debug, PartialEq)]
pub struct DnsQuery {
    pub name: String,
    pub qtype: DnsType,
    pub qclass: DnsClass,
}

impl DnsQuery {
    pub fn from_bytes(bytes: &[u8], offset: &mut usize) -> Result<Self, DnsQueryParseError> {
        let (name, new_offset) = parse_name(bytes, *offset)?;
        *offset = new_offset;

        check_dns_query_size(bytes, *offset, 4)?;

        let qtype = DnsType::new(u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]));
        let qclass = DnsClass::new(u16::from_be_bytes([bytes[*offset + 2], bytes[*offset + 3]]));
        *offset += 4;

        Ok(DnsQuery {
            name,
            qtype,
            qclass,
        })
    }
}

impl fmt::Display for DnsQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DnsQuery {{ name: {}, qtype: {}, qclass: {} }}",
            self.name, self.qtype, self.qclass
        )
    }
}

/// Parse un nom de domaine à partir d'un tableau d'octets, en suivant le format DNS.
///
/// # Arguments
/// - `bytes`: Référence à un tableau d'octets représentant le message DNS.
/// - `offset`: Position de départ dans `bytes` pour le parsing du nom de domaine.
///
/// # Returns
/// - `Ok((String, usize))` : Un tuple contenant le nom de domaine sous forme de chaîne de caractères
///   et la nouvelle valeur d'offset après le parsing.
/// - `Err(DnsQueryParseError)` : Retourne une erreur si les données sont insuffisantes ou
///   si une erreur de conversion UTF-8 survient.
///
/// # Errors
/// - `DnsQueryParseError::OutOfBoundParse` si les données ne contiennent pas assez d'octets pour un parsing correct.
/// - `DnsQueryParseError::Utf8Error` si les données ne sont pas des chaînes UTF-8 valides.
fn parse_name(bytes: &[u8], mut offset: usize) -> Result<(String, usize), DnsQueryParseError> {
    let mut labels = Vec::new(); // Stocke chaque label extrait du nom de domaine

    loop {
        // Vérifie que l'offset ne dépasse pas la longueur du tableau, sinon retourne une erreur
        if offset >= bytes.len() {
            return Err(DnsQueryParseError::OutOfBoundParse);
        }

        // Lit la longueur du prochain label (octet actuel)
        let len = bytes[offset] as usize;

        // Si la longueur est 0, le nom est terminé ; on avance l'offset d'un octet pour finir
        if len == 0 {
            offset += 1;
            break;
        }

        // Avance l'offset d'un octet pour pointer au début du label
        offset += 1;

        // Vérifie que le label complet est dans les limites de `bytes`, sinon retourne une erreur
        if offset + len > bytes.len() {
            return Err(DnsQueryParseError::OutOfBoundParse);
        }

        // Convertit le label en UTF-8 ; retourne une erreur si la conversion échoue
        let label = String::from_utf8(bytes[offset..offset + len].to_vec())?;
        labels.push(label); // Ajoute le label à la liste

        // Avance l'offset de la longueur du label pour traiter le suivant
        offset += len;
    }

    // Joint tous les labels avec des points pour former le nom complet
    let name = labels.join(".");
    Ok((name, offset)) // Retourne le nom et la nouvelle position de l'offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name() {
        let data = vec![
            0x03, 0x77, 0x77, 0x77, // "www"
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, // "google"
            0x03, 0x63, 0x6f, 0x6d, // "com"
            0x00, // Null terminator of the domain name
        ];
        let (name, offset) = parse_name(&data, 0).unwrap();
        assert_eq!(name, "www.google.com");
        assert_eq!(offset, 16);
    }

    #[test]
    fn test_parse_name_invalid_utf8() {
        // This data includes bytes that do not form valid UTF-8 sequences for labels.
        let data = vec![
            0x02, 0xFF, 0xFF, // Invalid UTF-8 bytes
            0x00, // Null terminator
        ];

        let result = parse_name(&data, 0);
        assert!(result.is_err());
        if let Err(DnsQueryParseError::Utf8Error(_)) = result {
            // Passed: The error is as expected.
        } else {
            panic!("Expected Utf8Error, but got {:?}", result);
        }
    }

    #[test]
    fn test_dns_query_from_bytes() {
        let data = vec![
            3, b'w', b'w', b'w', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, 0,
            1, 0, 1,
        ];
        let mut offset = 0;
        let query = DnsQuery::from_bytes(&data, &mut offset).unwrap();
        assert_eq!(query.name, "www.google.com");
        assert_eq!(query.qtype, DnsType(1));
        assert_eq!(query.qclass, DnsClass(1));
        assert_eq!(offset, 20);
    }

    #[test]
    fn test_dns_queries_from_bytes() {
        let data = vec![
            3, b'w', b'w', b'w', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, 0,
            1, 0, 1, 3, b'f', b'o', b'o', 3, b'b', b'a', b'r', 3, b'c', b'o', b'm', 0, 0, 2, 0, 1,
        ];
        let queries = DnsQueries::from_bytes(&data, 2).unwrap();
        assert_eq!(queries.queries.len(), 2);
        assert_eq!(queries.queries[0].name, "www.google.com");
        assert_eq!(queries.queries[0].qtype, DnsType(1));
        assert_eq!(queries.queries[0].qclass, DnsClass(1));
        assert_eq!(queries.queries[1].name, "foo.bar.com");
        assert_eq!(queries.queries[1].qtype, DnsType(2));
        assert_eq!(queries.queries[1].qclass, DnsClass(1));
    }
}
