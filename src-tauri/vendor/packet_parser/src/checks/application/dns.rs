// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::dns::{
    DnsFlagsError, DnsHeaderError, DnsPacketError, DnsQueryParseError,
};

pub const DNS_MINIMUM_SIZE: usize = 12;

pub fn check_dns_minimum_size(bytes: &[u8]) -> Result<(), DnsPacketError> {
    if bytes.len() < DNS_MINIMUM_SIZE {
        return Err(DnsPacketError::InsufficientData {
            expected: DNS_MINIMUM_SIZE,
            actual: bytes.len(),
        });
    }

    Ok(())
}

pub fn check_packet_length(bytes: &[u8]) -> Result<(), DnsHeaderError> {
    if bytes.len() < DNS_MINIMUM_SIZE {
        return Err(DnsHeaderError::PacketTooShort);
    }

    Ok(())
}

fn parse_counts(bytes: &[u8]) -> Result<[u16; 4], DnsHeaderError> {
    if bytes.len() < 8 {
        return Err(DnsHeaderError::PacketTooShort);
    }

    let questions_count = u16::from_be_bytes([bytes[0], bytes[1]]);
    let answers_count = u16::from_be_bytes([bytes[2], bytes[3]]);
    let authorities_count = u16::from_be_bytes([bytes[4], bytes[5]]);
    let additionals_count = u16::from_be_bytes([bytes[6], bytes[7]]);
    Ok([
        questions_count,
        answers_count,
        authorities_count,
        additionals_count,
    ])
}

pub fn validate_and_parse_count(bytes: &[u8]) -> Result<[u16; 4], DnsHeaderError> {
    let counts = parse_counts(bytes)?;

    if counts[0] == 0 && (counts[1] > 0 || counts[2] > 0 || counts[3] > 0) {
        return Err(DnsHeaderError::InvalidCounts);
    }

    Ok(counts)
}

/// Same as [`validate_and_parse_count`], but tolerates an empty question
/// section even when records are present.
///
/// Legitimate for mDNS (RFC 6762 §6): responders send unsolicited
/// announcements that never echo a question, unlike classic unicast DNS
/// where an answer without its matching question is a red flag.
pub(crate) fn parse_count_allow_empty_question(bytes: &[u8]) -> Result<[u16; 4], DnsHeaderError> {
    parse_counts(bytes)
}

pub fn check_dns_query_size(
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

pub fn check_dns_name_offset(bytes: &[u8], offset: usize) -> Result<(), DnsQueryParseError> {
    if offset >= bytes.len() {
        return Err(DnsQueryParseError::OutOfBoundParse);
    }

    Ok(())
}

pub fn check_dns_label_bounds(
    bytes: &[u8],
    offset: usize,
    label_len: usize,
) -> Result<(), DnsQueryParseError> {
    if offset + label_len > bytes.len() {
        return Err(DnsQueryParseError::OutOfBoundParse);
    }

    Ok(())
}

pub fn verify_dns_flags(flags: u16) -> Result<u16, DnsFlagsError> {
    let (qr, opcode, aa, tc, _rd, ra, z, rcode) = extract_dns_flags(flags);

    verify_z_field(z)?;
    verify_opcode(opcode)?;
    verify_rcode(rcode)?;
    verify_ra_in_query(qr, ra)?;

    if qr == 1 {
        verify_response_flags(opcode, aa, tc, rcode)?;
    }

    Ok(flags)
}

/// Validate the DNS header flags accepted on reception by mDNS.
///
/// RFC 6762 sections 18.3 and 18.11 require receivers to ignore messages with
/// non-zero OPCODE or RCODE. Its other header bits are explicitly ignored on
/// reception, so applying the stricter unicast-DNS validator here would reject
/// otherwise receivable mDNS traffic.
pub(crate) fn verify_mdns_flags(flags: u16) -> Result<u16, DnsFlagsError> {
    let (_, opcode, _, _, _, _, _, rcode) = extract_dns_flags(flags);

    if opcode != 0 {
        return Err(DnsFlagsError::InvalidOpcode(opcode));
    }
    if rcode != 0 {
        return Err(DnsFlagsError::InvalidRCode(rcode));
    }

    Ok(flags)
}

fn extract_dns_flags(flags: u16) -> (u16, u16, u16, u16, u16, u16, u16, u16) {
    let qr = (flags >> 15) & 0b1;
    let opcode = (flags >> 11) & 0b1111;
    let aa = (flags >> 10) & 0b1;
    let tc = (flags >> 9) & 0b1;
    let rd = (flags >> 8) & 0b1;
    let ra = (flags >> 7) & 0b1;
    let z = (flags >> 4) & 0b111;
    let rcode = flags & 0b1111;
    (qr, opcode, aa, tc, rd, ra, z, rcode)
}

fn verify_z_field(z: u16) -> Result<(), DnsFlagsError> {
    if z != 0 {
        return Err(DnsFlagsError::InvalidZField(z));
    }
    Ok(())
}

fn verify_opcode(opcode: u16) -> Result<(), DnsFlagsError> {
    if opcode > 5 {
        return Err(DnsFlagsError::InvalidOpcode(opcode));
    }
    Ok(())
}

fn verify_rcode(rcode: u16) -> Result<(), DnsFlagsError> {
    if rcode > 5 {
        return Err(DnsFlagsError::InvalidRCode(rcode));
    }
    Ok(())
}

fn verify_ra_in_query(qr: u16, ra: u16) -> Result<(), DnsFlagsError> {
    if qr == 0 && ra != 0 {
        return Err(DnsFlagsError::RaInQuery(ra));
    }
    Ok(())
}

fn verify_response_flags(opcode: u16, aa: u16, tc: u16, rcode: u16) -> Result<(), DnsFlagsError> {
    if opcode == 2 && (aa != 0 || tc != 0) {
        return Err(DnsFlagsError::AaTcInStatusResponse(aa, tc));
    }

    if rcode == 2 && aa != 0 {
        return Err(DnsFlagsError::AaInServerFailure(aa));
    }

    if rcode == 3 && aa != 1 {
        return Err(DnsFlagsError::AaInNameError(aa));
    }

    if rcode == 5 && aa != 0 {
        return Err(DnsFlagsError::AaInRefused(aa));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_z_field() {
        assert_eq!(verify_z_field(0), Ok(()));
        assert_eq!(verify_z_field(1), Err(DnsFlagsError::InvalidZField(1)));
    }

    #[test]
    fn test_verify_opcode() {
        assert_eq!(verify_opcode(0), Ok(()));
        assert_eq!(verify_opcode(5), Ok(()));
        assert_eq!(verify_opcode(6), Err(DnsFlagsError::InvalidOpcode(6)));
    }

    #[test]
    fn test_verify_rcode() {
        assert_eq!(verify_rcode(0), Ok(()));
        assert_eq!(verify_rcode(5), Ok(()));
        assert_eq!(verify_rcode(6), Err(DnsFlagsError::InvalidRCode(6)));
    }

    #[test]
    fn count_parsers_reject_short_slices_without_panicking() {
        let short_counts = [0_u8; 7];

        assert!(matches!(
            validate_and_parse_count(&short_counts),
            Err(DnsHeaderError::PacketTooShort)
        ));
        assert!(matches!(
            parse_count_allow_empty_question(&short_counts),
            Err(DnsHeaderError::PacketTooShort)
        ));
    }

    #[test]
    fn mdns_flags_require_zero_opcode_and_rcode() {
        assert_eq!(verify_mdns_flags(0x8400), Ok(0x8400));
        // AA/TC/RD/RA/Z/AD/CD are ignored on reception (RFC 6762
        // sections 18.4-18.10), including an unset AA bit in a response.
        assert_eq!(verify_mdns_flags(0x07f0), Ok(0x07f0));
        assert_eq!(verify_mdns_flags(0x8000), Ok(0x8000));
        assert_eq!(
            verify_mdns_flags(0x8800),
            Err(DnsFlagsError::InvalidOpcode(1))
        );
        assert_eq!(
            verify_mdns_flags(0x8001),
            Err(DnsFlagsError::InvalidRCode(1))
        );
    }
}
