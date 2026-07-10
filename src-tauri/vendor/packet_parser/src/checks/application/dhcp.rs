// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::dhcp::DhcpParseError;

pub const DHCP_MIN_LEN: usize = 236;

pub fn validate_dhcp_min_length(payload: &[u8]) -> Result<(), DhcpParseError> {
    if payload.len() < DHCP_MIN_LEN {
        return Err(DhcpParseError::PacketTooShort {
            expected: DHCP_MIN_LEN,
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn validate_operation(op: u8) -> Result<(), DhcpParseError> {
    if !(op == 1 || op == 2) {
        return Err(DhcpParseError::InvalidOperation { op });
    }

    Ok(())
}

pub fn validate_hardware_type(htype: u8) -> Result<(), DhcpParseError> {
    if htype != 1 {
        return Err(DhcpParseError::UnsupportedHardwareType { htype });
    }

    Ok(())
}

pub fn validate_hardware_address_length(hlen: u8) -> Result<(), DhcpParseError> {
    if hlen != 6 {
        return Err(DhcpParseError::InvalidHardwareAddressLength { hlen });
    }

    Ok(())
}

/// Magic cookie DHCP (RFC 2131) : premiers octets de la zone options.
pub const DHCP_MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];

/// Validates the DHCP magic cookie when an options area is present.
///
/// An empty options area is accepted (plain BOOTP, RFC 951). A non-empty
/// area must start with the RFC 2131 magic cookie: this keeps arbitrary
/// 236+-byte payloads from being misdetected as DHCP.
pub fn validate_magic_cookie(options: &[u8]) -> Result<(), DhcpParseError> {
    if options.is_empty() {
        return Ok(());
    }
    if options.len() < DHCP_MAGIC_COOKIE.len() || options[..4] != DHCP_MAGIC_COOKIE {
        return Err(DhcpParseError::InvalidMagicCookie);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dhcp_min_length_ok() {
        let payload = vec![0u8; DHCP_MIN_LEN];
        assert!(validate_dhcp_min_length(&payload).is_ok());

        let longer = vec![0u8; DHCP_MIN_LEN + 8];
        assert!(validate_dhcp_min_length(&longer).is_ok());
    }

    #[test]
    fn test_validate_dhcp_min_length_too_short() {
        let payload = vec![0u8; DHCP_MIN_LEN - 1];
        assert!(matches!(
            validate_dhcp_min_length(&payload),
            Err(DhcpParseError::PacketTooShort {
                expected: DHCP_MIN_LEN,
                actual: 235
            })
        ));

        assert!(matches!(
            validate_dhcp_min_length(&[]),
            Err(DhcpParseError::PacketTooShort {
                expected: DHCP_MIN_LEN,
                actual: 0
            })
        ));
    }

    #[test]
    fn test_validate_operation() {
        assert!(validate_operation(1).is_ok());
        assert!(validate_operation(2).is_ok());
        assert!(matches!(
            validate_operation(0),
            Err(DhcpParseError::InvalidOperation { op: 0 })
        ));
        assert!(matches!(
            validate_operation(3),
            Err(DhcpParseError::InvalidOperation { op: 3 })
        ));
    }

    #[test]
    fn test_validate_hardware_type() {
        assert!(validate_hardware_type(1).is_ok());
        assert!(matches!(
            validate_hardware_type(6),
            Err(DhcpParseError::UnsupportedHardwareType { htype: 6 })
        ));
    }

    #[test]
    fn test_validate_magic_cookie() {
        // Zone options vide : BOOTP pur, accepte
        assert!(validate_magic_cookie(&[]).is_ok());
        // Cookie RFC 2131 en tete : accepte
        assert!(validate_magic_cookie(&[0x63, 0x82, 0x53, 0x63, 0x35, 0x01, 0x01]).is_ok());
        // Trop court pour contenir le cookie : rejete
        assert!(matches!(
            validate_magic_cookie(&[0x63, 0x82]),
            Err(DhcpParseError::InvalidMagicCookie)
        ));
        // Mauvais cookie : rejete
        assert!(matches!(
            validate_magic_cookie(&[0xde, 0xad, 0xbe, 0xef, 0x00]),
            Err(DhcpParseError::InvalidMagicCookie)
        ));
    }

    #[test]
    fn test_validate_hardware_address_length() {
        assert!(validate_hardware_address_length(6).is_ok());
        assert!(matches!(
            validate_hardware_address_length(0),
            Err(DhcpParseError::InvalidHardwareAddressLength { hlen: 0 })
        ));
        assert!(matches!(
            validate_hardware_address_length(16),
            Err(DhcpParseError::InvalidHardwareAddressLength { hlen: 16 })
        ));
    }
}
