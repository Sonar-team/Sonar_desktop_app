// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::copt::CotpParseError,
    parse::application::protocols::copt::{CotpParameter, CotpPduType},
};

/// A COTP header always contains at least LI and the TPDU code. Individual
/// TPDU decoders enforce their larger fixed parts afterward.
pub const COTP_MIN_LENGTH: usize = 2;
pub const PARAM_TPDU_SIZE_OR_NUMBER: u8 = 0xC0;
pub const PARAM_SRC_TSAP: u8 = 0xC1;
pub const PARAM_DST_TSAP: u8 = 0xC2;
pub const PARAM_CHECKSUM: u8 = 0xC3;
pub const PARAM_CLEARING_INFO: u8 = 0xE0;
pub const PARAM_EOT: u8 = 0x80;

pub fn validate_parameter_code(pdu_type: CotpPduType, code: u8) -> Result<(), CotpParseError> {
    // RFC 905 section 13, the parameters added by later ISO 8073 editions,
    // and the two ATN checksum extensions decoded by Wireshark. RFC 905
    // explicitly permits a CR to ignore an undefined parameter. Every other
    // known TPDU uses its own allow-list; a parameter being globally defined
    // does not make it legal in an unrelated TPDU.
    let checksum = matches!(code, 0x08 | 0x09 | 0xC3);
    let allowed = match pdu_type {
        CotpPduType::ConnectionRequest | CotpPduType::Other(_) => true,
        CotpPduType::ConnectionConfirm => {
            checksum
                || matches!(
                    code,
                    0x85..=0x89 | 0x8B | 0xC0..=0xC2 | 0xC4..=0xC7 | 0xF0 | 0xF2
                )
        }
        CotpPduType::DisconnectRequest => checksum || code == 0xE0,
        CotpPduType::DisconnectConfirm
        | CotpPduType::Data
        | CotpPduType::ExpeditedData
        | CotpPduType::ExpeditedDataAcknowledgement => checksum,
        CotpPduType::DataAcknowledgement => checksum || matches!(code, 0x8A | 0x8C),
        CotpPduType::TpduError => checksum || code == 0xC1,
        CotpPduType::Reject => false,
    };

    if !allowed {
        return Err(CotpParseError::UnexpectedParameterCode {
            pdu: pdu_name(pdu_type),
            code,
        });
    }

    Ok(())
}

fn validate_exact_parameter_len(
    code: u8,
    actual: usize,
    expected: usize,
) -> Result<(), CotpParseError> {
    if actual != expected {
        return Err(CotpParseError::InvalidParameterValueLength {
            code,
            expected,
            actual,
        });
    }
    Ok(())
}

/// Validate the value length of every defined parameter whose size is fixed.
/// User-defined parameters (C5 and E0) intentionally remain opaque.
pub fn validate_parameter_value_len(
    _pdu_type: CotpPduType,
    code: u8,
    actual: usize,
) -> Result<(), CotpParseError> {
    match code {
        0x08 => validate_exact_parameter_len(code, actual, 4),
        0x09 | 0x85 | 0x87 | 0x8A | 0x8B | 0xC3 => validate_exact_parameter_len(code, actual, 2),
        0x86 => validate_exact_parameter_len(code, actual, 3),
        0x88 | 0x8C => validate_exact_parameter_len(code, actual, 8),
        0x89 if !matches!(actual, 12 | 24) => Err(CotpParseError::InvalidParameterLength {
            code,
            expected: "12 or 24",
            actual,
        }),
        0xC0 | 0xC4 | 0xC6 => validate_exact_parameter_len(code, actual, 1),
        0xF0 if !(1..=4).contains(&actual) => Err(CotpParseError::InvalidParameterLength {
            code,
            expected: "1 to 4",
            actual,
        }),
        0xF2 => validate_exact_parameter_len(code, actual, 4),
        _ => Ok(()),
    }
}

pub const fn pdu_name(pdu_type: CotpPduType) -> &'static str {
    match pdu_type {
        CotpPduType::ConnectionRequest => "CR",
        CotpPduType::ConnectionConfirm => "CC",
        CotpPduType::DisconnectRequest => "DR",
        CotpPduType::DisconnectConfirm => "DC",
        CotpPduType::Data => "DT",
        CotpPduType::ExpeditedData => "ED",
        CotpPduType::DataAcknowledgement => "AK",
        CotpPduType::ExpeditedDataAcknowledgement => "EA",
        CotpPduType::Reject => "RJ",
        CotpPduType::TpduError => "ER",
        CotpPduType::Other(_) => "unknown TPDU",
    }
}

/// Checked addition shared by every public COTP offset helper.
pub fn checked_offset(
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<usize, CotpParseError> {
    offset
        .checked_add(length)
        .ok_or(CotpParseError::LengthOverflow { context })
}

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

pub fn validate_length_indicator(length: u8) -> Result<(), CotpParseError> {
    if matches!(length, 0 | u8::MAX) {
        return Err(CotpParseError::InvalidLengthIndicator { length });
    }

    Ok(())
}

pub fn validate_fixed_header_len(
    declared_end: usize,
    expected: usize,
    pdu: &'static str,
) -> Result<(), CotpParseError> {
    if declared_end < expected {
        return Err(CotpParseError::FixedHeaderTooShort {
            pdu,
            expected,
            actual: declared_end,
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
    let header_end = checked_offset(offset, 2, "parameter header")?;
    if header_end > declared_end {
        return Err(CotpParseError::ParameterHeaderTruncated { offset });
    }

    Ok(())
}

pub fn validate_parameter_len(
    declared_end: usize,
    offset: usize,
    param_len: usize,
) -> Result<(), CotpParseError> {
    let value_start = checked_offset(offset, 2, "parameter value start")?;
    let value_end = checked_offset(value_start, param_len, "parameter value end")?;
    if value_end > declared_end {
        return Err(CotpParseError::ParameterLengthExceedsPacket {
            offset,
            declared: param_len,
            available: declared_end.saturating_sub(value_start),
        });
    }

    Ok(())
}

pub fn validate_cr_cc_class_options(
    class_option: u8,
    credit: u8,
) -> Result<(u8, bool, bool), CotpParseError> {
    let class = class_option >> 4;
    if class > 4 {
        return Err(CotpParseError::InvalidProtocolClass { class });
    }
    if class_option & 0x0C != 0 {
        return Err(CotpParseError::ReservedClassOptionBits {
            value: class_option,
        });
    }

    let extended_formats = class_option & 0x02 != 0;
    if extended_formats && class < 2 {
        return Err(CotpParseError::InvalidClassOption {
            class,
            option: "extended formats",
        });
    }

    let no_explicit_flow_control = class_option & 0x01 != 0;
    if no_explicit_flow_control && class != 2 {
        return Err(CotpParseError::InvalidClassOption {
            class,
            option: "no explicit flow control",
        });
    }

    if class <= 1 && credit != 0 {
        return Err(CotpParseError::InvalidInitialCredit { class, credit });
    }

    Ok((class, extended_formats, no_explicit_flow_control))
}

pub fn validate_cr_destination_reference(dst_ref: u16) -> Result<(), CotpParseError> {
    if dst_ref != 0 {
        return Err(CotpParseError::InvalidCrDestinationReference {
            destination_reference: dst_ref,
        });
    }

    Ok(())
}

pub fn validate_connection_references(
    pdu_type: CotpPduType,
    dst_ref: u16,
    src_ref: u16,
) -> Result<(), CotpParseError> {
    let pdu = pdu_name(pdu_type);
    if pdu_type == CotpPduType::ConnectionRequest {
        validate_cr_destination_reference(dst_ref)?;
    } else if pdu_type == CotpPduType::ConnectionConfirm && dst_ref == 0 {
        return Err(CotpParseError::InvalidConnectionReference {
            pdu,
            field: "destination",
            value: dst_ref,
        });
    }

    if src_ref == 0 {
        return Err(CotpParseError::InvalidConnectionReference {
            pdu,
            field: "source",
            value: src_ref,
        });
    }

    Ok(())
}

pub fn validate_disconnect_reason(reason: u8) -> Result<(), CotpParseError> {
    if !matches!(reason, 0..=3 | 0x80..=0x85 | 0x87 | 0x88 | 0x8A) {
        return Err(CotpParseError::InvalidDisconnectReason { reason });
    }

    Ok(())
}

pub fn validate_reject_cause(cause: u8) -> Result<(), CotpParseError> {
    if cause > 3 {
        return Err(CotpParseError::InvalidRejectCause { cause });
    }

    Ok(())
}

pub fn validate_sequence_number_high_bit(
    pdu: &'static str,
    number: u32,
) -> Result<(), CotpParseError> {
    if number & 0x8000_0000 != 0 {
        return Err(CotpParseError::ReservedSequenceNumberBit { pdu });
    }
    Ok(())
}

pub fn validate_normal_sequence_number_high_bit(
    pdu: &'static str,
    number: u8,
) -> Result<(), CotpParseError> {
    validate_sequence_number_high_bit(pdu, u32::from(number) << 24)
}

pub fn validate_extended_credit_code(pdu: &'static str, code: u8) -> Result<(), CotpParseError> {
    if code & 0x0F != 0 {
        return Err(CotpParseError::InvalidExtendedCreditCode { pdu, code });
    }
    Ok(())
}

pub fn validate_tpdu_size_code(code: u8) -> Result<(), CotpParseError> {
    if !(0x07..=0x0D).contains(&code) {
        return Err(CotpParseError::InvalidTpduSizeCode { code });
    }

    Ok(())
}

pub fn validate_user_data(
    pdu: &'static str,
    class: Option<u8>,
    actual: usize,
    maximum: usize,
) -> Result<(), CotpParseError> {
    if matches!(class, Some(0)) && actual != 0 {
        return Err(CotpParseError::UserDataNotAllowed { pdu, class: 0 });
    }
    if actual > maximum {
        return Err(CotpParseError::UserDataTooLong {
            pdu,
            maximum,
            actual,
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
    _offset: usize,
    param_data: &'a [u8],
) -> Result<CotpParameter<'a>, CotpParseError> {
    validate_parameter_code(pdu_type, param_type)?;
    validate_parameter_value_len(pdu_type, param_type, param_data.len())?;

    let param = match param_type {
        0x08 => CotpParameter::AtnExtendedChecksum32(u32::from_be_bytes([
            param_data[0],
            param_data[1],
            param_data[2],
            param_data[3],
        ])),
        0x09 => {
            CotpParameter::AtnExtendedChecksum16(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_TPDU_SIZE_OR_NUMBER
            if matches!(
                pdu_type,
                CotpPduType::ConnectionRequest | CotpPduType::ConnectionConfirm
            ) =>
        {
            validate_tpdu_size_code(param_data[0])?;
            CotpParameter::TpduSize(param_data[0])
        }
        PARAM_SRC_TSAP
            if matches!(
                pdu_type,
                CotpPduType::ConnectionRequest | CotpPduType::ConnectionConfirm
            ) && param_data.len() == 2 =>
        {
            CotpParameter::SrcTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_DST_TSAP
            if matches!(
                pdu_type,
                CotpPduType::ConnectionRequest | CotpPduType::ConnectionConfirm
            ) && param_data.len() == 2 =>
        {
            CotpParameter::DstTsap(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_CHECKSUM => {
            CotpParameter::Checksum(u16::from_be_bytes([param_data[0], param_data[1]]))
        }
        PARAM_CLEARING_INFO if pdu_type == CotpPduType::DisconnectRequest => {
            CotpParameter::DisconnectAdditionalInfo(param_data)
        }
        PARAM_SRC_TSAP if pdu_type == CotpPduType::TpduError => {
            CotpParameter::InvalidTpdu(param_data)
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
    validate_length_indicator(data[0])?;

    Ok((data[0], CotpPduType::from(data[1])))
}

/// Vérifie et extrait les champs de connexion d'un PDU CR/CC :
/// références destination et source, classe, puis les drapeaux
/// « extended formats » et « no explicit flow control ».
pub fn extract_connection_fields(
    data: &[u8],
    offset: usize,
    declared_end: usize,
) -> Result<(u16, u16, u8, bool, bool), CotpParseError> {
    let expected = checked_offset(offset, 5, "CR/CC fixed header")?;
    validate_connection_header_len(declared_end, expected)?;
    // Garde hors contrat : from_bytes a déjà vérifié
    // data.len() >= declared_end (validate_declared_len), cette branche est
    // donc inatteignable depuis le parseur. Elle garantit que les
    // indexations ci-dessous sont bornées même sur un appel direct.
    validate_declared_len(data.len(), declared_end)?;

    let fields = &data[offset..expected];
    let dst_ref = u16::from_be_bytes([fields[0], fields[1]]);
    let src_ref = u16::from_be_bytes([fields[2], fields[3]]);
    let (class, extended_formats, no_explicit_flow_control) =
        validate_cr_cc_class_options(fields[4], 0)?;

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
    let length_offset = checked_offset(offset, 1, "parameter length field")?;
    let param_len = data[length_offset] as usize;

    validate_parameter_len(declared_end, offset, param_len)?;

    let value_start = checked_offset(offset, 2, "parameter value start")?;
    let value_end = checked_offset(value_start, param_len, "parameter value end")?;
    let param_data = &data[value_start..value_end];
    let param = parse_cotp_parameter(pdu_type, param_type, offset, param_data)?;

    Ok((param, value_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_and_declared_lengths_are_checked() {
        assert!(validate_min_len(&[0x01, 0x42]).is_ok());
        assert!(matches!(
            validate_min_len(&[0x01]),
            Err(CotpParseError::PacketTooShort {
                expected: 2,
                actual: 1
            })
        ));
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
    fn length_indicator_rejects_reserved_values() {
        assert!(validate_length_indicator(1).is_ok());
        for length in [0, u8::MAX] {
            assert_eq!(
                validate_length_indicator(length),
                Err(CotpParseError::InvalidLengthIndicator { length })
            );
        }
    }

    #[test]
    fn fixed_header_guards_report_the_expected_layout() {
        assert!(validate_connection_header_len(7, 7).is_ok());
        assert!(matches!(
            validate_connection_header_len(5, 7),
            Err(CotpParseError::ConnectionHeaderTooShort {
                expected: 7,
                actual: 5
            })
        ));
        assert!(matches!(
            validate_fixed_header_len(5, 6, "DC"),
            Err(CotpParseError::FixedHeaderTooShort {
                pdu: "DC",
                expected: 6,
                actual: 5
            })
        ));
    }

    #[test]
    fn parameter_bounds_are_checked_without_overflow() {
        assert!(validate_parameter_header(9, 7).is_ok());
        assert!(matches!(
            validate_parameter_header(8, 7),
            Err(CotpParseError::ParameterHeaderTruncated { offset: 7 })
        ));
        assert!(validate_parameter_len(10, 6, 2).is_ok());
        assert!(matches!(
            validate_parameter_len(10, 6, 3),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 6,
                declared: 3,
                available: 2
            })
        ));
        assert!(matches!(
            checked_offset(usize::MAX, 1, "test"),
            Err(CotpParseError::LengthOverflow { context: "test" })
        ));
        assert!(matches!(
            validate_parameter_header(usize::MAX, usize::MAX),
            Err(CotpParseError::LengthOverflow {
                context: "parameter header"
            })
        ));
        assert!(matches!(
            validate_parameter_len(usize::MAX, usize::MAX - 2, 1),
            Err(CotpParseError::LengthOverflow {
                context: "parameter value end"
            })
        ));
    }

    #[test]
    fn legacy_tpdu_number_helper_rejects_empty_values() {
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
    fn class_options_and_initial_credit_follow_rfc_905() {
        assert_eq!(
            validate_cr_cc_class_options(0x00, 0).unwrap(),
            (0, false, false)
        );
        assert_eq!(
            validate_cr_cc_class_options(0x23, 15).unwrap(),
            (2, true, true)
        );
        assert_eq!(
            validate_cr_cc_class_options(0x42, 1).unwrap(),
            (4, true, false)
        );

        assert!(matches!(
            validate_cr_cc_class_options(0x50, 0),
            Err(CotpParseError::InvalidProtocolClass { class: 5 })
        ));
        assert!(matches!(
            validate_cr_cc_class_options(0x24, 0),
            Err(CotpParseError::ReservedClassOptionBits { value: 0x24 })
        ));
        assert!(matches!(
            validate_cr_cc_class_options(0x12, 0),
            Err(CotpParseError::InvalidClassOption { class: 1, .. })
        ));
        assert!(matches!(
            validate_cr_cc_class_options(0x31, 0),
            Err(CotpParseError::InvalidClassOption { class: 3, .. })
        ));
        assert!(matches!(
            validate_cr_cc_class_options(0x10, 1),
            Err(CotpParseError::InvalidInitialCredit {
                class: 1,
                credit: 1
            })
        ));
    }

    #[test]
    fn connection_references_reject_zero_and_cr_destination_values() {
        assert!(validate_connection_references(CotpPduType::ConnectionRequest, 0, 1).is_ok());
        assert!(validate_connection_references(CotpPduType::ConnectionConfirm, 1, 2).is_ok());
        assert!(matches!(
            validate_connection_references(CotpPduType::ConnectionRequest, 1, 2),
            Err(CotpParseError::InvalidCrDestinationReference { .. })
        ));
        assert!(matches!(
            validate_connection_references(CotpPduType::ConnectionRequest, 0, 0),
            Err(CotpParseError::InvalidConnectionReference {
                pdu: "CR",
                field: "source",
                ..
            })
        ));
        assert!(matches!(
            validate_connection_references(CotpPduType::ConnectionConfirm, 0, 1),
            Err(CotpParseError::InvalidConnectionReference {
                pdu: "CC",
                field: "destination",
                ..
            })
        ));
    }

    #[test]
    fn reason_cause_and_sequence_fields_are_validated() {
        for reason in [0, 1, 2, 3, 0x80, 0x85, 0x87, 0x88, 0x8A] {
            assert!(validate_disconnect_reason(reason).is_ok());
        }
        for reason in [4, 0x86, 0x89, 0xFF] {
            assert_eq!(
                validate_disconnect_reason(reason),
                Err(CotpParseError::InvalidDisconnectReason { reason })
            );
        }
        for cause in 0..=3 {
            assert!(validate_reject_cause(cause).is_ok());
        }
        assert!(matches!(
            validate_reject_cause(4),
            Err(CotpParseError::InvalidRejectCause { cause: 4 })
        ));

        assert!(validate_normal_sequence_number_high_bit("EA", 0x7F).is_ok());
        assert!(matches!(
            validate_normal_sequence_number_high_bit("EA", 0x80),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "EA" })
        ));
        assert!(validate_sequence_number_high_bit("AK", 0x7FFF_FFFF).is_ok());
        assert!(matches!(
            validate_sequence_number_high_bit("AK", 0x8000_0000),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "AK" })
        ));
        assert!(validate_extended_credit_code("RJ", 0x50).is_ok());
        assert!(matches!(
            validate_extended_credit_code("RJ", 0x51),
            Err(CotpParseError::InvalidExtendedCreditCode {
                pdu: "RJ",
                code: 0x51
            })
        ));
    }

    #[test]
    fn tpdu_size_codes_are_exactly_the_rfc_range() {
        for code in 0x07..=0x0D {
            assert!(validate_tpdu_size_code(code).is_ok());
        }
        for code in [0x00, 0x06, 0x0E, 0xFF] {
            assert_eq!(
                validate_tpdu_size_code(code),
                Err(CotpParseError::InvalidTpduSizeCode { code })
            );
        }
    }

    #[test]
    fn parameter_catalog_is_specific_to_each_tpdu() {
        for code in [
            0x08, 0x09, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8B, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5,
            0xC6, 0xC7, 0xF0, 0xF2,
        ] {
            assert!(validate_parameter_code(CotpPduType::ConnectionConfirm, code).is_ok());
        }
        assert!(validate_parameter_code(CotpPduType::ConnectionRequest, 0xAA).is_ok());
        assert!(matches!(
            validate_parameter_code(CotpPduType::ConnectionConfirm, 0xAA),
            Err(CotpParseError::UnexpectedParameterCode {
                pdu: "CC",
                code: 0xAA
            })
        ));

        for pdu in [
            CotpPduType::DisconnectConfirm,
            CotpPduType::Data,
            CotpPduType::ExpeditedData,
            CotpPduType::ExpeditedDataAcknowledgement,
        ] {
            for code in [0x08, 0x09, 0xC3] {
                assert!(validate_parameter_code(pdu, code).is_ok());
            }
            assert!(validate_parameter_code(pdu, 0xC0).is_err());
        }
        for code in [0x08, 0x09, 0x8A, 0x8C, 0xC3] {
            assert!(validate_parameter_code(CotpPduType::DataAcknowledgement, code).is_ok());
        }
        assert!(validate_parameter_code(CotpPduType::DataAcknowledgement, 0x8B).is_err());
        for code in [0x08, 0x09, 0xC3, 0xE0] {
            assert!(validate_parameter_code(CotpPduType::DisconnectRequest, code).is_ok());
        }
        for code in [0x08, 0x09, 0xC1, 0xC3] {
            assert!(validate_parameter_code(CotpPduType::TpduError, code).is_ok());
        }
        assert!(validate_parameter_code(CotpPduType::Reject, 0xC3).is_err());
    }

    #[test]
    fn fixed_parameter_lengths_are_not_silently_accepted() {
        for (code, length) in [
            (0x08, 4),
            (0x09, 2),
            (0x85, 2),
            (0x86, 3),
            (0x87, 2),
            (0x88, 8),
            (0x89, 12),
            (0x89, 24),
            (0x8A, 2),
            (0x8B, 2),
            (0x8C, 8),
            (0xC0, 1),
            (0xC3, 2),
            (0xC4, 1),
            (0xC6, 1),
            (0xF0, 1),
            (0xF0, 4),
            (0xF2, 4),
        ] {
            assert!(
                validate_parameter_value_len(CotpPduType::ConnectionConfirm, code, length).is_ok(),
                "code {code:#04x}, length {length}"
            );
        }

        for (code, length) in [
            (0x08, 2),
            (0x09, 4),
            (0x85, 1),
            (0x86, 2),
            (0x87, 3),
            (0x88, 7),
            (0x89, 13),
            (0x8A, 1),
            (0x8B, 8),
            (0x8C, 2),
            (0xC0, 2),
            (0xC3, 1),
            (0xC4, 2),
            (0xC6, 0),
            (0xF0, 0),
            (0xF0, 5),
            (0xF2, 3),
        ] {
            assert!(
                validate_parameter_value_len(CotpPduType::ConnectionConfirm, code, length).is_err(),
                "code {code:#04x}, length {length}"
            );
        }

        // RFC 905 intentionally leaves TSAP-ID and user-defined lengths open.
        for code in [0xC1, 0xC2, 0xC5, 0xC7, 0xE0] {
            assert!(validate_parameter_value_len(CotpPduType::ConnectionConfirm, code, 3).is_ok());
        }
    }

    #[test]
    fn parameters_are_classified_zero_copy_after_validation() {
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC0, 7, &[0x09]).unwrap(),
            CotpParameter::TpduSize(0x09)
        );
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC1, 7, &[0x01, 0x00]).unwrap(),
            CotpParameter::SrcTsap(0x0100)
        );
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC2, 7, &[0x01, 0x02]).unwrap(),
            CotpParameter::DstTsap(0x0102)
        );
        let tsap = [1, 2, 3];
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC1, 7, &tsap).unwrap(),
            CotpParameter::Other(0xC1, &tsap)
        );
        assert_eq!(
            parse_cotp_parameter(CotpPduType::Data, 0xC3, 5, &[0x12, 0x34]).unwrap(),
            CotpParameter::Checksum(0x1234)
        );
        let clearing = [0xAA, 0xBB, 0xCC];
        assert_eq!(
            parse_cotp_parameter(CotpPduType::DisconnectRequest, 0xE0, 7, &clearing).unwrap(),
            CotpParameter::DisconnectAdditionalInfo(&clearing)
        );
        let rejected = [0xF0, 0x80, 0x00];
        assert_eq!(
            parse_cotp_parameter(CotpPduType::TpduError, 0xC1, 5, &rejected).unwrap(),
            CotpParameter::InvalidTpdu(&rejected)
        );
        let unknown = [0xDE, 0xAD];
        assert_eq!(
            parse_cotp_parameter(CotpPduType::ConnectionRequest, 0xAA, 7, &unknown).unwrap(),
            CotpParameter::Other(0xAA, &unknown)
        );
        assert!(matches!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC0, 7, &[0x09, 0x0A]),
            Err(CotpParseError::InvalidParameterValueLength { code: 0xC0, .. })
        ));
        assert!(matches!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xC0, 7, &[0x0E]),
            Err(CotpParseError::InvalidTpduSizeCode { code: 0x0E })
        ));
        assert!(matches!(
            parse_cotp_parameter(CotpPduType::ConnectionConfirm, 0xAA, 7, &[]),
            Err(CotpParseError::UnexpectedParameterCode { .. })
        ));
    }

    #[test]
    fn length_and_pdu_type_extraction_accepts_two_byte_raw_headers() {
        let data = [0x06, 0xE0, 0x00, 0x01, 0x00, 0x02, 0x00];
        assert_eq!(
            extract_length_and_pdu_type(&data).unwrap(),
            (0x06, CotpPduType::ConnectionRequest)
        );
        assert_eq!(
            extract_length_and_pdu_type(&[0x01, 0x42]).unwrap(),
            (1, CotpPduType::Other(0x42))
        );
        assert!(matches!(
            extract_length_and_pdu_type(&[0x02]),
            Err(CotpParseError::PacketTooShort {
                expected: 2,
                actual: 1
            })
        ));
        for length in [0, u8::MAX] {
            assert!(matches!(
                extract_length_and_pdu_type(&[length, 0x42]),
                Err(CotpParseError::InvalidLengthIndicator { .. })
            ));
        }
    }

    #[test]
    fn connection_field_extraction_uses_the_rfc_option_bits() {
        let data = [0x06, 0xE0, 0x00, 0x00, 0x00, 0x02, 0x23];
        assert_eq!(
            extract_connection_fields(&data, 2, 7).unwrap(),
            (0x0000, 0x0002, 2, true, true)
        );
        let data = [0x04, 0xE0, 0x00, 0x01, 0x00];
        assert!(matches!(
            extract_connection_fields(&data, 2, 5),
            Err(CotpParseError::ConnectionHeaderTooShort {
                expected: 7,
                actual: 5
            })
        ));
        let data = [0x06, 0xE0, 0x00];
        assert!(matches!(
            extract_connection_fields(&data, 2, 7),
            Err(CotpParseError::LengthExceedsPacket {
                declared: 7,
                actual: 3
            })
        ));
        assert!(matches!(
            extract_connection_fields(&[], usize::MAX, 0),
            Err(CotpParseError::LengthOverflow {
                context: "CR/CC fixed header"
            })
        ));
    }

    #[test]
    fn parameter_extraction_is_bounded_and_advances_exactly() {
        let data = [0x09, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC0, 0x01, 0x09];
        let (param, next_offset) =
            extract_parameter(&data, 7, 10, CotpPduType::ConnectionConfirm).unwrap();
        assert_eq!(param, CotpParameter::TpduSize(0x09));
        assert_eq!(next_offset, 10);

        let data = [0x07, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC1];
        assert!(matches!(
            extract_parameter(&data, 7, 8, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::ParameterHeaderTruncated { offset: 7 })
        ));

        let data = [0x08, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC1, 0x0A];
        assert!(matches!(
            extract_parameter(&data, 7, 9, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::ParameterLengthExceedsPacket {
                offset: 7,
                declared: 10,
                available: 0
            })
        ));

        let data = [0x09, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00];
        assert!(matches!(
            extract_parameter(&data, 7, 10, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::LengthExceedsPacket {
                declared: 10,
                actual: 7
            })
        ));
        assert!(matches!(
            extract_parameter(&[0], usize::MAX, 0, CotpPduType::ConnectionConfirm),
            Err(CotpParseError::LengthOverflow {
                context: "parameter header"
            })
        ));
    }

    #[test]
    fn user_data_limits_are_explicit() {
        assert!(validate_user_data("CR", Some(0), 0, 32).is_ok());
        assert!(matches!(
            validate_user_data("CR", Some(0), 1, 32),
            Err(CotpParseError::UserDataNotAllowed {
                pdu: "CR",
                class: 0
            })
        ));
        assert!(validate_user_data("CC", Some(1), 32, 32).is_ok());
        assert!(matches!(
            validate_user_data("CC", Some(1), 33, 32),
            Err(CotpParseError::UserDataTooLong {
                pdu: "CC",
                maximum: 32,
                actual: 33
            })
        ));
    }
}
