// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use super::{DecodedLink, LinkDecoder};
use crate::{
    LinkLayer, LinkLayerError, LinkType, LinuxArphrdType, LinuxCookedPacketType, LinuxSllLink,
    ParseError,
};

const HEADER_LEN: usize = 16;
const ADDRESS_OFFSET: usize = 6;
const ADDRESS_SLOT_LEN: usize = 8;
const PROTOCOL_OFFSET: usize = 14;

/// Decoder for the fixed 16-byte LINKTYPE_LINUX_SLL v1 cooked header.
pub(super) struct LinuxSllDecoder;

impl LinkDecoder for LinuxSllDecoder {
    #[inline(always)]
    fn decode<'a>(bytes: &'a [u8]) -> Result<DecodedLink<'a>, ParseError> {
        if bytes.len() < HEADER_LEN {
            return Err(LinkLayerError::Truncated {
                link_type: LinkType::LINUX_SLL,
                required: HEADER_LEN,
                actual: bytes.len(),
            }
            .into());
        }

        let packet_type = LinuxCookedPacketType(u16::from_be_bytes([bytes[0], bytes[1]]));
        let hardware_type = LinuxArphrdType(u16::from_be_bytes([bytes[2], bytes[3]]));
        let address_length = u16::from_be_bytes([bytes[4], bytes[5]]);
        let available_address_len = usize::from(address_length).min(ADDRESS_SLOT_LEN);
        let source_address = if available_address_len == 0 {
            None
        } else {
            Some(&bytes[ADDRESS_OFFSET..ADDRESS_OFFSET + available_address_len])
        };
        let protocol = u16::from_be_bytes([bytes[PROTOCOL_OFFSET], bytes[PROTOCOL_OFFSET + 1]]);
        let payload = &bytes[HEADER_LEN..];

        Ok(DecodedLink::new(LinkLayer::linux_sll(LinuxSllLink::new(
            packet_type,
            hardware_type,
            address_length,
            source_address,
            protocol,
            payload,
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkProtocol, ParseError};

    fn header(protocol: u16) -> [u8; HEADER_LEN] {
        let mut bytes = [0_u8; HEADER_LEN];
        bytes[0..2].copy_from_slice(&4_u16.to_be_bytes());
        bytes[2..4].copy_from_slice(&1_u16.to_be_bytes());
        bytes[4..6].copy_from_slice(&6_u16.to_be_bytes());
        bytes[6..12].copy_from_slice(&[0, 1, 2, 3, 4, 5]);
        bytes[PROTOCOL_OFFSET..HEADER_LEN].copy_from_slice(&protocol.to_be_bytes());
        bytes
    }

    #[test]
    fn decodes_big_endian_fields_and_zero_copy_slices() {
        let mut bytes = header(0x0800).to_vec();
        bytes.extend_from_slice(&[0x45, 0, 0, 20]);

        let decoded = LinuxSllDecoder::decode(&bytes).unwrap();
        let (layer, protocol, payload) = decoded.into_parts();
        let sll = layer.as_linux_sll().unwrap();

        assert_eq!(layer.link_type(), LinkType::LINUX_SLL);
        assert_eq!(protocol, NetworkProtocol::Ipv4);
        assert_eq!(sll.packet_type, LinuxCookedPacketType::OUTGOING);
        assert_eq!(sll.hardware_type, LinuxArphrdType::ETHERNET);
        assert_eq!(sll.address_length, 6);
        assert_eq!(sll.source_address, Some(&[0, 1, 2, 3, 4, 5][..]));
        assert_eq!(
            sll.source_address.unwrap().as_ptr(),
            bytes[ADDRESS_OFFSET..].as_ptr()
        );
        assert_eq!(sll.protocol, 0x0800);
        assert_eq!(payload, &bytes[HEADER_LEN..]);
        assert_eq!(payload.as_ptr(), bytes[HEADER_LEN..].as_ptr());
        assert_eq!(sll.payload.as_ptr(), bytes[HEADER_LEN..].as_ptr());
    }

    #[test]
    fn every_short_header_is_a_structured_link_truncation() {
        let bytes = header(0x0800);

        for len in 0..HEADER_LEN {
            assert!(matches!(
                LinuxSllDecoder::decode(&bytes[..len]),
                Err(ParseError::InvalidLinkLayer(LinkLayerError::Truncated {
                    link_type: LinkType::LINUX_SLL,
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
            let decoded = LinuxSllDecoder::decode(&bytes).unwrap();
            let (layer, protocol, _) = decoded.into_parts();

            assert_eq!(protocol, expected);
            assert!(layer.as_ethernet().is_none());
            assert!(layer.as_raw_ip().is_none());
            assert!(layer.as_ieee80211().is_none());
        }
    }

    #[test]
    fn address_length_controls_the_view_and_padding_is_not_flow_identity() {
        let address = [1, 2, 3, 4, 5, 6, 7, 8];

        for declared_length in [0_u16, 4, 6, 8] {
            let mut bytes = header(0x893a);
            bytes[4..6].copy_from_slice(&declared_length.to_be_bytes());
            bytes[6..14].copy_from_slice(&address);

            let decoded = LinuxSllDecoder::decode(&bytes).unwrap();
            let (layer, _, _) = decoded.into_parts();
            let sll = layer.as_linux_sll().unwrap();
            let expected = if declared_length == 0 {
                None
            } else {
                Some(&address[..usize::from(declared_length)])
            };

            assert_eq!(sll.source_address, expected);
            assert!(!sll.address_is_truncated());
        }

        let mut first = header(0x893a);
        first[4..6].copy_from_slice(&4_u16.to_be_bytes());
        first[6..14].copy_from_slice(&address);
        let mut second = first;
        second[10..14].copy_from_slice(&[90, 91, 92, 93]);

        let (first, _, _) = LinuxSllDecoder::decode(&first).unwrap().into_parts();
        let (second, _, _) = LinuxSllDecoder::decode(&second).unwrap().into_parts();
        assert_eq!(first, second);
        assert_eq!(
            serde_json::to_value(first).unwrap(),
            serde_json::to_value(second).unwrap()
        );
    }

    #[test]
    fn preserves_future_values_and_caps_the_address_view_to_the_wire_slot() {
        for declared_length in [9_u16, u16::MAX] {
            let mut bytes = header(0x9999);
            bytes[0..2].copy_from_slice(&0x1234_u16.to_be_bytes());
            bytes[2..4].copy_from_slice(&0xffff_u16.to_be_bytes());
            bytes[4..6].copy_from_slice(&declared_length.to_be_bytes());
            bytes[6..14].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

            let decoded = LinuxSllDecoder::decode(&bytes).unwrap();
            let (layer, protocol, _) = decoded.into_parts();
            let sll = layer.as_linux_sll().unwrap();

            assert_eq!(sll.packet_type, LinuxCookedPacketType(0x1234));
            assert_eq!(sll.hardware_type, LinuxArphrdType(0xffff));
            assert_eq!(sll.address_length, declared_length);
            assert_eq!(sll.source_address, Some(&[1, 2, 3, 4, 5, 6, 7, 8][..]));
            assert!(sll.address_is_truncated());
            assert_eq!(protocol, NetworkProtocol::Other(0x9999));
        }
    }
}
