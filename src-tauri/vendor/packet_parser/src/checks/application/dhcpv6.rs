// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::dhcpv6::Dhcpv6PacketParseError;

pub const DHCPV6_MIN_CLIENT_SERVER_LEN: usize = 4;

/// Checks that the payload holds at least the DHCPv6 fixed header
/// (message type + transaction id, 4 bytes).
pub fn validate_dhcpv6_min_length(payload: &[u8]) -> Result<(), Dhcpv6PacketParseError> {
    if payload.len() < DHCPV6_MIN_CLIENT_SERVER_LEN {
        return Err(Dhcpv6PacketParseError::PacketLength);
    }
    Ok(())
}

/// Checks that the message type is a known DHCPv6 client/server message type (1..=13).
///
/// Kept for API compatibility; delegates to [`extract_message_type`].
pub fn validate_dhcpv6_message_type(message_type: u8) -> Result<(), Dhcpv6PacketParseError> {
    extract_message_type(&message_type)?;
    Ok(())
}

/// Checks the message type byte (known DHCPv6 client/server message types are
/// 1..=13) and returns it, ready to be placed in the parsed structure.
pub fn extract_message_type(message_type: &u8) -> Result<u8, Dhcpv6PacketParseError> {
    if !(1..=13).contains(message_type) {
        return Err(Dhcpv6PacketParseError::MessageType {
            message_type: *message_type,
        });
    }
    Ok(*message_type)
}

/// Checks that the slice holds the 3 transaction-id bytes and assembles them
/// into a zero-padded `u32` (the transaction id is 24 bits on the wire).
///
/// The length guard is unreachable after [`validate_dhcpv6_min_length`]; it is
/// kept as a local bound check, like `extract_root_delay` in the NTP model.
pub fn extract_transaction_id(payload: &[u8]) -> Result<u32, Dhcpv6PacketParseError> {
    if payload.len() < 3 {
        return Err(Dhcpv6PacketParseError::PacketLength);
    }
    Ok(u32::from_be_bytes([0, payload[0], payload[1], payload[2]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dhcpv6_min_length() {
        assert!(validate_dhcpv6_min_length(&[0u8; 4]).is_ok());
        assert!(matches!(
            validate_dhcpv6_min_length(&[0u8; 3]),
            Err(Dhcpv6PacketParseError::PacketLength)
        ));
    }

    #[test]
    fn test_extract_message_type_valid_bounds() {
        assert_eq!(extract_message_type(&1), Ok(1));
        assert_eq!(extract_message_type(&13), Ok(13));
    }

    #[test]
    fn test_extract_message_type_invalid() {
        assert!(matches!(
            extract_message_type(&0),
            Err(Dhcpv6PacketParseError::MessageType { message_type: 0 })
        ));
        assert!(matches!(
            extract_message_type(&14),
            Err(Dhcpv6PacketParseError::MessageType { message_type: 14 })
        ));
    }

    #[test]
    fn test_validate_dhcpv6_message_type_delegates() {
        assert!(validate_dhcpv6_message_type(1).is_ok());
        assert!(matches!(
            validate_dhcpv6_message_type(14),
            Err(Dhcpv6PacketParseError::MessageType { message_type: 14 })
        ));
    }

    #[test]
    fn test_extract_transaction_id() {
        assert_eq!(extract_transaction_id(&[0x12, 0x34, 0x56]), Ok(0x0012_3456));
    }

    #[test]
    fn test_extract_transaction_id_too_short() {
        assert!(matches!(
            extract_transaction_id(&[0x12, 0x34]),
            Err(Dhcpv6PacketParseError::PacketLength)
        ));
    }
}
