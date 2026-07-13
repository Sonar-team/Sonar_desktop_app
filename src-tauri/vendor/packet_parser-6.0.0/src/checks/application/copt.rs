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
}
