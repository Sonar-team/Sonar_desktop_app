use crate::errors::internet::arp::ArpError;
use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// ARP Packet Structure
///
/// Represents an Address Resolution Protocol (ARP) packet
#[derive(Debug, PartialEq)]
pub struct ArpPacket {
    /// Hardware type (e.g., 1 for Ethernet)
    pub hardware_type: u16,
    /// Protocol type (e.g., 0x0800 for IPv4)
    pub protocol_type: u16,
    /// Length of hardware address in bytes (e.g., 6 for Ethernet)
    pub hardware_len: u8,
    /// Length of protocol address in bytes (e.g., 4 for IPv4)
    pub protocol_len: u8,
    /// Operation (1 for request, 2 for reply)
    pub operation: u16,
    /// Sender hardware address (MAC address for Ethernet)
    pub sender_hardware_addr: [u8; 6],
    /// Sender protocol address (IPv4 or IPv6 address)
    pub sender_protocol_addr: IpAddr,
    /// Target hardware address (MAC address for Ethernet)
    pub target_hardware_addr: [u8; 6],
    /// Target protocol address (IPv4 or IPv6 address)
    pub target_protocol_addr: IpAddr,
}

impl TryFrom<&[u8]> for ArpPacket {
    type Error = ArpError;

    /// Attempts to parse a byte slice into an Arp packet
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the ARP packet
    ///
    /// # Returns
    /// * `Result<Arp, InternetError>` - The parsed ARP packet or an error
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Minimum ARP packet size is 28 bytes (for IPv4 over Ethernet)
        let min_len = 28;

        if data.len() < min_len {
            return Err(ArpError::InvalidLength {
                expected: min_len,
                actual: data.len(),
            });
        }

        let hardware_type = u16::from_be_bytes([data[0], data[1]]);
        if hardware_type != 1 {
            return Err(ArpError::UnsupportedHardwareType(hardware_type));
        }
        let protocol_type = u16::from_be_bytes([data[2], data[3]]);
        let hardware_len = data[4];
        if hardware_len != 6 {
            return Err(ArpError::InvalidHardwareLength {
                expected: 6,
                actual: hardware_len,
            });
        }
        let protocol_len = data[5];

        // Check for minimum length based on protocol type
        let min_len = 8 + (2 * hardware_len as usize) + (2 * protocol_len as usize);
        if data.len() < min_len {
            return Err(ArpError::InvalidLength {
                expected: min_len,
                actual: data.len(),
            });
        }
        let operation = u16::from_be_bytes([data[6], data[7]]);
        if operation != 1 && operation != 2 {
            return Err(ArpError::UnsupportedOperation(operation));
        }

        let sender_hardware_addr = [data[8], data[9], data[10], data[11], data[12], data[13]];

        let sender_protocol_addr = match protocol_type {
            0x0800 => {
                if protocol_len != 4 {
                    return Err(ArpError::InvalidProtocolLength {
                        expected: 4,
                        actual: protocol_len,
                    });
                }
                IpAddr::V4(Ipv4Addr::new(data[14], data[15], data[16], data[17]))
            }
            0x86DD => {
                if protocol_len != 16 {
                    return Err(ArpError::InvalidProtocolLength {
                        expected: 16,
                        actual: protocol_len,
                    });
                }
                let mut addr = [0u8; 16];
                addr.copy_from_slice(&data[14..30]);
                IpAddr::V6(Ipv6Addr::from(addr))
            }
            _ => return Err(ArpError::UnsupportedProtocolType(protocol_type)),
        };

        let target_hardware_addr = [
            data[14 + protocol_len as usize],
            data[15 + protocol_len as usize],
            data[16 + protocol_len as usize],
            data[17 + protocol_len as usize],
            data[18 + protocol_len as usize],
            data[19 + protocol_len as usize],
        ];

        let target_protocol_addr = match protocol_type {
            0x0800 => IpAddr::V4(Ipv4Addr::new(
                data[20 + protocol_len as usize],
                data[21 + protocol_len as usize],
                data[22 + protocol_len as usize],
                data[23 + protocol_len as usize],
            )),
            0x86DD => {
                let start = 20 + protocol_len as usize;
                let end = start + 16;
                let mut addr = [0u8; 16];
                addr.copy_from_slice(&data[start..end]);
                IpAddr::V6(Ipv6Addr::from(addr))
            }
            _ => unreachable!(), // We already checked protocol_type above
        };

        Ok(ArpPacket {
            hardware_type,
            protocol_type,
            hardware_len,
            protocol_len,
            operation,
            sender_hardware_addr,
            sender_protocol_addr,
            target_hardware_addr,
            target_protocol_addr,
        })
    }
}
