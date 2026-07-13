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

pub fn validate_and_parse_count(bytes: &[u8]) -> Result<[u16; 4], DnsHeaderError> {
    let questions_count = u16::from_be_bytes([bytes[0], bytes[1]]);
    let answers_count = u16::from_be_bytes([bytes[2], bytes[3]]);
    let authorities_count = u16::from_be_bytes([bytes[4], bytes[5]]);
    let additionals_count = u16::from_be_bytes([bytes[6], bytes[7]]);

    if questions_count == 0 && (answers_count > 0 || authorities_count > 0 || additionals_count > 0)
    {
        return Err(DnsHeaderError::InvalidCounts);
    }

    Ok([
        questions_count,
        answers_count,
        authorities_count,
        additionals_count,
    ])
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
}
