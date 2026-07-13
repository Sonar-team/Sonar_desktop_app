// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use super::{DecodedLink, LinkDecoder};
use crate::{
    LinkLayer, LinkLayerError, LinkType, LinuxArphrdType, LinuxCookedPacketType, LinuxSll2Link,
    ParseError,
};

const HEADER_LEN: usize = 20;
const RESERVED_OFFSET: usize = 2;
const INTERFACE_INDEX_OFFSET: usize = 4;
const HARDWARE_TYPE_OFFSET: usize = 8;
const PACKET_TYPE_OFFSET: usize = 10;
const ADDRESS_LENGTH_OFFSET: usize = 11;
const ADDRESS_OFFSET: usize = 12;
const ADDRESS_SLOT_LEN: usize = 8;

/// Decoder for the fixed 20-byte LINKTYPE_LINUX_SLL2 cooked header.
pub(super) struct LinuxSll2Decoder;

impl LinkDecoder for LinuxSll2Decoder {
    #[inline(always)]
    fn decode<'a>(bytes: &'a [u8]) -> Result<DecodedLink<'a>, ParseError> {
        if bytes.len() < HEADER_LEN {
            return Err(LinkLayerError::Truncated {
                link_type: LinkType::LINUX_SLL2,
                required: HEADER_LEN,
                actual: bytes.len(),
            }
            .into());
        }

        let protocol = u16::from_be_bytes([bytes[0], bytes[1]]);
        let reserved_mbz = u16::from_be_bytes([bytes[RESERVED_OFFSET], bytes[RESERVED_OFFSET + 1]]);
        let interface_index = u32::from_be_bytes([
            bytes[INTERFACE_INDEX_OFFSET],
            bytes[INTERFACE_INDEX_OFFSET + 1],
            bytes[INTERFACE_INDEX_OFFSET + 2],
            bytes[INTERFACE_INDEX_OFFSET + 3],
        ]);
        let hardware_type = LinuxArphrdType(u16::from_be_bytes([
            bytes[HARDWARE_TYPE_OFFSET],
            bytes[HARDWARE_TYPE_OFFSET + 1],
        ]));
        let packet_type = LinuxCookedPacketType(u16::from(bytes[PACKET_TYPE_OFFSET]));
        let address_length = bytes[ADDRESS_LENGTH_OFFSET];
        let available_address_len = usize::from(address_length).min(ADDRESS_SLOT_LEN);
        let source_address = if available_address_len == 0 {
            None
        } else {
            Some(&bytes[ADDRESS_OFFSET..ADDRESS_OFFSET + available_address_len])
        };
        let payload = &bytes[HEADER_LEN..];

        Ok(DecodedLink::new(LinkLayer::linux_sll2(LinuxSll2Link {
            protocol,
            reserved_mbz,
            interface_index,
            hardware_type,
            packet_type,
            address_length,
            source_address,
            payload,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkProtocol, ParseError};

    fn header(protocol: u16) -> [u8; HEADER_LEN] {
        let mut bytes = [0_u8; HEADER_LEN];
        bytes[0..2].copy_from_slice(&protocol.to_be_bytes());
        bytes[INTERFACE_INDEX_OFFSET..HARDWARE_TYPE_OFFSET]
            .copy_from_slice(&0x0102_0304_u32.to_be_bytes());
        bytes[HARDWARE_TYPE_OFFSET..PACKET_TYPE_OFFSET]
            .copy_from_slice(&LinuxArphrdType::LOOPBACK.0.to_be_bytes());
        bytes[PACKET_TYPE_OFFSET] = LinuxCookedPacketType::OUTGOING.0 as u8;
        bytes[ADDRESS_LENGTH_OFFSET] = 6;
        bytes[ADDRESS_OFFSET..ADDRESS_OFFSET + 6].copy_from_slice(&[0, 1, 2, 3, 4, 5]);
        bytes
    }

    #[test]
    fn decodes_wire_order_and_zero_copy_slices() {
        let mut bytes = header(0x0800).to_vec();
        bytes[RESERVED_OFFSET..INTERFACE_INDEX_OFFSET].copy_from_slice(&0x1234_u16.to_be_bytes());
        bytes.extend_from_slice(&[0x45, 0, 0, 20]);

        let decoded = LinuxSll2Decoder::decode(&bytes).unwrap();
        let (layer, protocol, payload) = decoded.into_parts();
        let sll2 = layer.as_linux_sll2().unwrap();

        assert_eq!(layer.link_type(), LinkType::LINUX_SLL2);
        assert_eq!(protocol, NetworkProtocol::Ipv4);
        assert_eq!(sll2.protocol, 0x0800);
        assert_eq!(sll2.reserved_mbz, 0x1234);
        assert!(!sll2.reserved_is_zero());
        assert_eq!(sll2.interface_index, 0x0102_0304);
        assert_eq!(sll2.hardware_type, LinuxArphrdType::LOOPBACK);
        assert_eq!(sll2.packet_type, LinuxCookedPacketType::OUTGOING);
        assert_eq!(sll2.address_length, 6);
        assert_eq!(sll2.source_address, Some(&[0, 1, 2, 3, 4, 5][..]));
        assert_eq!(
            sll2.source_address.unwrap().as_ptr(),
            bytes[ADDRESS_OFFSET..].as_ptr()
        );
        assert_eq!(payload, &bytes[HEADER_LEN..]);
        assert_eq!(payload.as_ptr(), bytes[HEADER_LEN..].as_ptr());
        assert_eq!(sll2.payload.as_ptr(), bytes[HEADER_LEN..].as_ptr());
    }

    #[test]
    fn every_short_header_is_a_structured_link_truncation() {
        let bytes = header(0x0800);

        for len in 0..HEADER_LEN {
            assert!(matches!(
                LinuxSll2Decoder::decode(&bytes[..len]),
                Err(ParseError::InvalidLinkLayer(LinkLayerError::Truncated {
                    link_type: LinkType::LINUX_SLL2,
                    required: HEADER_LEN,
                    actual,
                })) if actual == len
            ));
        }
    }

    #[test]
    fn maps_ip_arp_and_unknown_protocols_without_fabricating_ethernet() {
        for (wire, expected) in [
            (0x0800, NetworkProtocol::Ipv4),
            (0x86dd, NetworkProtocol::Ipv6),
            (0x0806, NetworkProtocol::Arp),
            (0x8892, NetworkProtocol::Profinet),
            (0x893a, NetworkProtocol::Other(0x893a)),
            (0x0004, NetworkProtocol::Other(0x0004)),
        ] {
            let bytes = header(wire);
            let decoded = LinuxSll2Decoder::decode(&bytes).unwrap();
            let (layer, protocol, _) = decoded.into_parts();

            assert_eq!(protocol, expected);
            assert!(layer.as_ethernet().is_none());
            assert!(layer.as_raw_ip().is_none());
            assert!(layer.as_linux_sll().is_none());
            assert!(layer.as_ieee80211().is_none());
        }
    }

    #[test]
    fn address_length_controls_the_view_and_padding_is_not_flow_identity() {
        let address = [1, 2, 3, 4, 5, 6, 7, 8];

        for declared_length in [0_u8, 4, 6, 8] {
            let mut bytes = header(0x893a);
            bytes[ADDRESS_LENGTH_OFFSET] = declared_length;
            bytes[ADDRESS_OFFSET..HEADER_LEN].copy_from_slice(&address);

            let decoded = LinuxSll2Decoder::decode(&bytes).unwrap();
            let (layer, _, _) = decoded.into_parts();
            let sll2 = layer.as_linux_sll2().unwrap();
            let expected = if declared_length == 0 {
                None
            } else {
                Some(&address[..usize::from(declared_length)])
            };

            assert_eq!(sll2.source_address, expected);
            assert!(!sll2.address_is_truncated());
        }

        let mut first = header(0x893a);
        first[ADDRESS_LENGTH_OFFSET] = 4;
        first[ADDRESS_OFFSET..HEADER_LEN].copy_from_slice(&address);
        let mut second = first;
        second[ADDRESS_OFFSET + 4..HEADER_LEN].copy_from_slice(&[90, 91, 92, 93]);

        let (first, _, _) = LinuxSll2Decoder::decode(&first).unwrap().into_parts();
        let (second, _, _) = LinuxSll2Decoder::decode(&second).unwrap().into_parts();
        assert_eq!(first, second);
        assert_eq!(
            serde_json::to_value(first).unwrap(),
            serde_json::to_value(second).unwrap()
        );
    }

    #[test]
    fn preserves_future_values_and_caps_the_address_view_to_the_wire_slot() {
        for declared_length in [9_u8, u8::MAX] {
            let mut bytes = header(0x9999);
            bytes[RESERVED_OFFSET..INTERFACE_INDEX_OFFSET]
                .copy_from_slice(&0x1234_u16.to_be_bytes());
            bytes[INTERFACE_INDEX_OFFSET..HARDWARE_TYPE_OFFSET]
                .copy_from_slice(&u32::MAX.to_be_bytes());
            bytes[HARDWARE_TYPE_OFFSET..PACKET_TYPE_OFFSET]
                .copy_from_slice(&0xffff_u16.to_be_bytes());
            bytes[PACKET_TYPE_OFFSET] = u8::MAX;
            bytes[ADDRESS_LENGTH_OFFSET] = declared_length;
            bytes[ADDRESS_OFFSET..HEADER_LEN].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

            let decoded = LinuxSll2Decoder::decode(&bytes).unwrap();
            let (layer, protocol, _) = decoded.into_parts();
            let sll2 = layer.as_linux_sll2().unwrap();

            assert_eq!(sll2.reserved_mbz, 0x1234);
            assert!(!sll2.reserved_is_zero());
            assert_eq!(sll2.interface_index, u32::MAX);
            assert_eq!(sll2.hardware_type, LinuxArphrdType(0xffff));
            assert_eq!(sll2.packet_type, LinuxCookedPacketType(0xff));
            assert_eq!(sll2.address_length, declared_length);
            assert_eq!(sll2.source_address, Some(&[1, 2, 3, 4, 5, 6, 7, 8][..]));
            assert!(sll2.address_is_truncated());
            assert_eq!(protocol, NetworkProtocol::Other(0x9999));
        }
    }
}
