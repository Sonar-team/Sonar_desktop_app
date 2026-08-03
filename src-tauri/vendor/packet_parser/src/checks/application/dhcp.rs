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

/// Checks the DHCP operation code (must be BOOTREQUEST=1 or BOOTREPLY=2)
/// and returns it, ready to be placed in the packet struct.
pub fn extract_operation(op: u8) -> Result<u8, DhcpParseError> {
    validate_operation(op)?;
    Ok(op)
}

/// Checks the hardware type (only Ethernet, htype=1, is supported) and
/// returns it.
pub fn extract_hardware_type(htype: u8) -> Result<u8, DhcpParseError> {
    validate_hardware_type(htype)?;
    Ok(htype)
}

/// Checks the hardware address length (must be 6, Ethernet MAC) and
/// returns it.
pub fn extract_hardware_address_length(hlen: u8) -> Result<u8, DhcpParseError> {
    validate_hardware_address_length(hlen)?;
    Ok(hlen)
}

/// Returns the hops counter. RFC 2131 puts no constraint on this field
/// (relay agents may set any value), so no validation applies.
pub fn extract_hops(hops: u8) -> Result<u8, DhcpParseError> {
    Ok(hops)
}

/// Checks that the slice holds the 4 bytes of the transaction id (xid)
/// and returns it as a big-endian `u32`.
pub fn extract_xid(payload: &[u8]) -> Result<u32, DhcpParseError> {
    if payload.len() < 4 {
        return Err(DhcpParseError::PacketTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok(u32::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

/// Checks that the slice holds the 2 bytes of the seconds field (secs)
/// and returns it as a big-endian `u16`.
pub fn extract_secs(payload: &[u8]) -> Result<u16, DhcpParseError> {
    if payload.len() < 2 {
        return Err(DhcpParseError::PacketTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_be_bytes([payload[0], payload[1]]))
}

/// Checks that the slice holds the 2 bytes of the flags field and
/// returns it as a big-endian `u16`.
pub fn extract_flags(payload: &[u8]) -> Result<u16, DhcpParseError> {
    if payload.len() < 2 {
        return Err(DhcpParseError::PacketTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_be_bytes([payload[0], payload[1]]))
}

/// Checks that the slice holds the 4 bytes of an IPv4 address field
/// (ciaddr, yiaddr, siaddr or giaddr) and returns them as `[u8; 4]`.
pub fn extract_ipv4_addr(payload: &[u8]) -> Result<[u8; 4], DhcpParseError> {
    if payload.len() < 4 {
        return Err(DhcpParseError::PacketTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok([payload[0], payload[1], payload[2], payload[3]])
}

/// Checks that the slice is exactly the 16-byte client hardware address
/// area (chaddr) and returns it as a borrowed array (zero-copy).
pub fn extract_chaddr(payload: &[u8]) -> Result<&[u8; 16], DhcpParseError> {
    payload
        .try_into()
        .map_err(|_| DhcpParseError::PacketTooShort {
            expected: 16,
            actual: payload.len(),
        })
}

/// Checks that the slice is exactly the 64-byte server host name area
/// (sname) and returns it as a borrowed array (zero-copy).
pub fn extract_sname(payload: &[u8]) -> Result<&[u8; 64], DhcpParseError> {
    payload
        .try_into()
        .map_err(|_| DhcpParseError::PacketTooShort {
            expected: 64,
            actual: payload.len(),
        })
}

/// Checks that the slice is exactly the 128-byte boot file name area
/// (file) and returns it as a borrowed array (zero-copy).
pub fn extract_file(payload: &[u8]) -> Result<&[u8; 128], DhcpParseError> {
    payload
        .try_into()
        .map_err(|_| DhcpParseError::PacketTooShort {
            expected: 128,
            actual: payload.len(),
        })
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

    #[test]
    fn test_extract_operation() {
        assert_eq!(extract_operation(1), Ok(1));
        assert_eq!(extract_operation(2), Ok(2));
        assert!(matches!(
            extract_operation(3),
            Err(DhcpParseError::InvalidOperation { op: 3 })
        ));
    }

    #[test]
    fn test_extract_hardware_type() {
        assert_eq!(extract_hardware_type(1), Ok(1));
        assert!(matches!(
            extract_hardware_type(6),
            Err(DhcpParseError::UnsupportedHardwareType { htype: 6 })
        ));
    }

    #[test]
    fn test_extract_hardware_address_length() {
        assert_eq!(extract_hardware_address_length(6), Ok(6));
        assert!(matches!(
            extract_hardware_address_length(0),
            Err(DhcpParseError::InvalidHardwareAddressLength { hlen: 0 })
        ));
    }

    #[test]
    fn test_extract_hops() {
        assert_eq!(extract_hops(0), Ok(0));
        assert_eq!(extract_hops(255), Ok(255));
    }

    #[test]
    fn test_extract_xid() {
        assert_eq!(extract_xid(&[0x39, 0x03, 0xF3, 0x26]), Ok(0x3903F326));
        assert!(matches!(
            extract_xid(&[0x39, 0x03]),
            Err(DhcpParseError::PacketTooShort {
                expected: 4,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_extract_secs_and_flags() {
        assert_eq!(extract_secs(&[0x00, 0x0A]), Ok(10));
        assert_eq!(extract_flags(&[0x80, 0x00]), Ok(0x8000));
        assert!(matches!(
            extract_secs(&[0x00]),
            Err(DhcpParseError::PacketTooShort {
                expected: 2,
                actual: 1
            })
        ));
        assert!(matches!(
            extract_flags(&[]),
            Err(DhcpParseError::PacketTooShort {
                expected: 2,
                actual: 0
            })
        ));
    }

    #[test]
    fn test_extract_ipv4_addr() {
        assert_eq!(extract_ipv4_addr(&[192, 168, 0, 10]), Ok([192, 168, 0, 10]));
        assert!(matches!(
            extract_ipv4_addr(&[192, 168]),
            Err(DhcpParseError::PacketTooShort {
                expected: 4,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_extract_chaddr() {
        let bytes = [0xABu8; 16];
        assert_eq!(extract_chaddr(&bytes), Ok(&bytes));
        assert!(matches!(
            extract_chaddr(&bytes[..8]),
            Err(DhcpParseError::PacketTooShort {
                expected: 16,
                actual: 8
            })
        ));
    }

    #[test]
    fn test_extract_sname_and_file() {
        let sname = [0u8; 64];
        assert_eq!(extract_sname(&sname), Ok(&sname));
        assert!(matches!(
            extract_sname(&sname[..10]),
            Err(DhcpParseError::PacketTooShort {
                expected: 64,
                actual: 10
            })
        ));

        let file = [0u8; 128];
        assert_eq!(extract_file(&file), Ok(&file));
        assert!(matches!(
            extract_file(&file[..0]),
            Err(DhcpParseError::PacketTooShort {
                expected: 128,
                actual: 0
            })
        ));
    }
}
