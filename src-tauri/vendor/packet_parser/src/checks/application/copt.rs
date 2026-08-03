// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::copt::CotpParseError,
    parse::application::protocols::copt::{CotpParameter, CotpPduType},
};

pub const COTP_MIN_LENGTH: usize = 3;
pub const PARAM_TPDU_SIZE_OR_NUMBER: u8 = 0xC0;
pub const PARAM_SRC_TSAP: u8 = 0xC1;
pub const PARAM_DST_TSAP: u8 = 0xC2;
pub const PARAM_EOT: u8 = 0x80;

pub fn validate_min_len(data: &[u8]) -> Result<(), CotpParseError> {
    if data.len() < COTP_MIN_LENGTH {
        return Err(CotpParseError::PacketTooShort {
            expected: COTP_MIN_LENGTH,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_declared_len(data_len: usize, declared_end: usize) -> Result<(), CotpParseError> {
    if data_len < declared_end {
        return Err(CotpParseError::LengthExceedsPacket {
            declared: declared_end,
            actual: data_len,
        });
    }

    Ok(())
}

pub fn validate_connection_header_len(
    declared_end: usize,
    expected: usize,
) -> Result<(), CotpParseError> {
    if declared_end < expected {
        return Err(CotpParseError::ConnectionHeaderTooShort {
            expected,
            actual: declared_end,
        });
    }

    Ok(())
}

pub fn validate_parameter_header(declared_end: usize, offset: usize) -> Result<(), CotpParseError> {
    if offset + 1 >= declared_end {
        return Err(CotpParseError::ParameterHeaderTruncated { offset });
    }

    Ok(())
}

pub fn validate_parameter_len(
    declared_end: usize,
    offset: usize,
    param_len: usize,
) -> Result<(), CotpParseError> {
    if offset + 2 + param_len > declared_end {
        return Err(CotpParseError::ParameterLengthExceedsPacket {
            offset,
            declared: param_len,
            available: declared_end.saturating_sub(offset + 2),
        });
    }

    Ok(())
}

/// Rejette un paramètre TPDU-number (0xC0 sur un DT) sans donnée.
///
/// Réutilise le variant [`CotpParseError::ParameterLengthExceedsPacket`] avec
/// `declared: 1, available: 0`, à lire comme « au moins 1 octet attendu,
/// 0 disponible » (la longueur déclarée sur le wire est 0). Un variant dédié
/// serait plus fidèle mais changerait l'API publique.
pub fn validate_tpdu_number_not_empty(offset: usize, len: usize) -> Result<(), CotpParseError> {
    if len == 0 {
        return Err(CotpParseError::ParameterLengthExceedsPacket {
            offset,
            declared: 1,
            available: 0,
        });
    }

    Ok(())
}

/// Classifie un paramètre COTP en validant sa longueur selon son type.
///
/// La slice `param_data` est empruntée au paquet original (zero-copy) : les
/// variantes non reconnues la conservent telle quelle dans
/// [`CotpParameter::Other`].
pub fn parse_cotp_parameter<'a>(
    pdu_type: CotpPduType,
    param_type: u8,
    offset: usize,
    param_data: &'a [u8],
) -> Result<CotpParameter<'a>, CotpParseError> {
    let param = match param_type {
        PARAM_TPDU_SIZE_OR_NUMBER => {
            // TPDU size (CR/CC) ou TPDU number (DT)
            if pdu_type == CotpPduType::Data {
                validate_tpdu_number_not_empty(offset, param_data.len())?;
                CotpParameter::TpduNumber(param_data[0])
            } else if param_data.len() == 1 {
                CotpParameter::TpduSize(param_data[0])
            } else {
                CotpParameter::Other(param_type, param_data)
            }
        }
        PARAM_SRC_TSAP if param_data.len() == 2 => {
            CotpParameter::SrcTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_DST_TSAP if param_data.len() == 2 => {
            CotpParameter::DstTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_EOT if pdu_type == CotpPduType::Data && param_data.is_empty() => {
            CotpParameter::Eot(true)
        }
        _ => CotpParameter::Other(param_type, param_data),
    };

    Ok(param)
}

/// Vérifie les deux premiers octets du paquet et retourne le champ longueur
/// (octet 0) et le type de PDU (octet 1) prêts à être placés.
///
/// Re-vérifie la longueur minimale : la branche d'erreur est inatteignable
/// après [`validate_min_len`] et ne protège qu'un appel hors contrat.
pub fn extract_length_and_pdu_type(data: &[u8]) -> Result<(u8, CotpPduType), CotpParseError> {
    validate_min_len(data)?;

    Ok((data[0], CotpPduType::from(data[1])))
}

/// Vérifie et extrait les champs de connexion d'un PDU CR/CC/DR/DC/ER :
/// références destination et source, classe, puis les drapeaux
/// « extended formats » et « no explicit flow control ».
pub fn extract_connection_fields(
    data: &[u8],
    offset: usize,
    declared_end: usize,
) -> Result<(u16, u16, u8, bool, bool), CotpParseError> {
    let expected = offset + 5;
    validate_connection_header_len(declared_end, expected)?;
    // Garde hors contrat : from_bytes a déjà vérifié
    // data.len() >= declared_end (validate_declared_len), cette branche est
    // donc inatteignable depuis le parseur. Elle garantit que les
    // indexations ci-dessous sont bornées même sur un appel direct.
    validate_declared_len(data.len(), declared_end)?;

    let dst_ref = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let src_ref = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
    let class = data[offset + 4] >> 4;
    let extended_formats = (data[offset + 4] & 0x04) != 0;
    let no_explicit_flow_control = (data[offset + 4] & 0x01) != 0;

    Ok((
        dst_ref,
        src_ref,
        class,
        extended_formats,
        no_explicit_flow_control,
    ))
}

/// Vérifie et extrait le paramètre COTP situé à `offset` : en-tête
/// type/longueur, longueur déclarée du paramètre, puis classification via
/// [`parse_cotp_parameter`]. Retourne le paramètre typé (zero-copy) et
/// l'offset du paramètre suivant.
pub fn extract_parameter<'a>(
    data: &'a [u8],
    offset: usize,
    declared_end: usize,
    pdu_type: CotpPduType,
) -> Result<(CotpParameter<'a>, usize), CotpParseError> {
    // Garde hors contrat : inatteignable depuis from_bytes qui a déjà
    // vérifié data.len() >= declared_end ; borne les indexations ci-dessous
    // pour un appel direct.
    validate_declared_len(data.len(), declared_end)?;
    validate_parameter_header(declared_end, offset)?;

    let param_type = data[offset];
    let param_len = data[offset + 1] as usize;

    validate_parameter_len(declared_end, offset, param_len)?;

    let param_data = &data[offset + 2..offset + 2 + param_len];
    let param = parse_cotp_parameter(pdu_type, param_type, offset, param_data)?;

    Ok((param, offset + 2 + param_len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_min_len() {
        assert!(validate_min_len(&[0x02, 0xF0, 0x80]).is_ok());
        assert!(matches!(
            validate_min_len(&[0x02, 0xF0]),
            Err(CotpParseError::PacketTooShort {
                expected: 3,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_validate_declared_len() {
        assert!(validate_declared_len(10, 10).is_ok());
        assert!(validate_declared_len(10, 5).is_ok());
        assert!(matches!(
            validate_declared_len(5, 10),
            Err(CotpParseError::LengthExceedsPacket {
                declared: 10,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_validate_connection_header_len() {
        assert!(validate_connection_header_len(7, 7).is_ok());
        assert!(matches!(
            validate_connection_header_len(5, 7),
            Err(CotpParseError::ConnectionHeaderTooShort {
                expected: 7,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_validate_parameter_header() {
        // Deux octets disponibles pour type + longueur : ok
        assert!(validate_parameter_header(9, 7).is_ok());
        // Un seul octet restant : tronqué
        assert!(matches!(
            validate_parameter_header(8, 7),
            Err(CotpParseError::ParameterHeaderTruncated { offset: 7 })
        ));
    }

    #[test]
    fn test_validate_parameter_len() {
        assert!(validate_parameter_len(10, 6, 2).is_ok());
        assert!(matches!(
            validate_parameter_len(10, 6, 3),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 6,
                declared: 3,
                available: 2
            })
        ));
    }

    #[test]
    fn test_validate_tpdu_number_not_empty() {
        assert!(validate_tpdu_number_not_empty(2, 1).is_ok());
        assert!(matches!(
            validate_tpdu_number_not_empty(2, 0),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 2,
                declared: 1,
                available: 0
            })
        ));
    }

    #[test]
    fn test_parse_cotp_parameter_classification() {
        // 0xC0 sur un DT : TPDU number
        assert_eq!(
            parse_cotp_parameter(CotpPduType::Data, 0xC0, 2, &[0x05]).unwrap(),
            CotpParameter::TpduNumber(5)
        );
        // 0xC0 sur un CC avec 1 octet : TPDU size
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC0, 7, &[0x09]).unwrap(),
            CotpParameter::TpduSize(0x09)
        );
        // 0xC0 sur un CC avec 2 octets : Other
        let raw = [0x09, 0x0A];
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC0, 7, &raw).unwrap(),
            CotpParameter::Other(0xC0, &raw[..])
        );
        // TSAP source et destination
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC1, 7, &[0x01, 0x00]).unwrap(),
            CotpParameter::SrcTsap(0x0100)
        );
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC2, 7, &[0x01, 0x02]).unwrap(),
            CotpParameter::DstTsap(0x0102)
        );
        // EOT sur un DT sans donnée
        assert_eq!(
            parse_cotp_parameter(CotpPduType::Data, 0x80, 5, &[]).unwrap(),
            CotpParameter::Eot(true)
        );
        // EOT hors DT : Other
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0x80, 7, &[]).unwrap(),
            CotpParameter::Other(0x80, &[][..])
        );
    }

    #[test]
    fn test_parse_cotp_parameter_empty_tpdu_number() {
        assert!(matches!(
            parse_cotp_parameter(CotpPduType::Data, 0xC0, 2, &[]),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 2,
                declared: 1,
                available: 0
            })
        ));
    }

    #[test]
    fn test_extract_length_and_pdu_type() {
        let data = [0x06, 0xE0, 0x00, 0x01, 0x00, 0x02, 0x00];
        assert_eq!(
            extract_length_and_pdu_type(&data).unwrap(),
            (0x06, CotpPduType::ConnectionRequest)
        );
        assert!(matches!(
            extract_length_and_pdu_type(&[0x02, 0xF0]),
            Err(CotpParseError::PacketTooShort {
                expected: 3,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_extract_connection_fields() {
        // CR : refs, classe 4, extended formats et no explicit flow control.
        let data = [0x06, 0xE0, 0x00, 0x01, 0x00, 0x02, 0x45];
        assert_eq!(
            extract_connection_fields(&data, 2, 7).unwrap(),
            (0x0001, 0x0002, 4, true, true)
        );
    }

    #[test]
    fn test_extract_connection_fields_header_too_short() {
        // declared_end = 5 : trop court pour refs + classe (7 attendus).
        let data = [0x04, 0xE0, 0x00, 0x01, 0x00];
        assert!(matches!(
            extract_connection_fields(&data, 2, 5),
            Err(CotpParseError::ConnectionHeaderTooShort {
                expected: 7,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_extract_connection_fields_out_of_contract_guard() {
        // Appel hors contrat : declared_end dépasse la slice réelle.
        let data = [0x06, 0xE0, 0x00];
        assert!(matches!(
            extract_connection_fields(&data, 2, 7),
            Err(CotpParseError::LengthExceedsPacket {
                declared: 7,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_extract_parameter_tpdu_size() {
        // CC : paramètre 0xC0 (TPDU size) à l'offset 7, declared_end = 10.
        let data = [0x09, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC0, 0x01, 0x09];
        let (param, next_offset) =
            extract_parameter(&data, 7, 10, CotpPduType::ConnectionConfirm).unwrap();
        assert_eq!(param, CotpParameter::TpduSize(0x09));
        assert_eq!(next_offset, 10);
    }

    #[test]
    fn test_extract_parameter_header_truncated() {
        // Un seul octet restant : impossible de lire type + longueur.
        let data = [0x07, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC1];
        assert!(matches!(
            extract_parameter(&data, 7, 8, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::ParameterHeaderTruncated { offset: 7 })
        ));
    }

    #[test]
    fn test_extract_parameter_length_exceeds_packet() {
        // Le paramètre annonce 10 octets absents.
        let data = [0x08, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC1, 0x0A];
        assert!(matches!(
            extract_parameter(&data, 7, 9, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 7,
                declared: 10,
                available: 0
            })
        ));
    }

    #[test]
    fn test_extract_parameter_out_of_contract_guard() {
        // Appel hors contrat : declared_end dépasse la slice réelle.
        let data = [0x09, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00];
        assert!(matches!(
            extract_parameter(&data, 7, 10, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::LengthExceedsPacket {
                declared: 10,
                actual: 7
            })
        ));
    }
}
