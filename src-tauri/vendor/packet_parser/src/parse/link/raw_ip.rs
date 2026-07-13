// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use super::{DecodedLink, LinkDecoder};
use crate::{LinkLayer, LinkLayerError, LinkType, ParseError};

/// Decoder for LINKTYPE_RAW, whose packet bytes start directly with IPv4/IPv6.
pub(super) struct RawIpDecoder;

impl LinkDecoder for RawIpDecoder {
    #[inline(always)]
    fn decode<'a>(bytes: &'a [u8]) -> Result<DecodedLink<'a>, ParseError> {
        let first = bytes.first().copied().ok_or(LinkLayerError::Truncated {
            link_type: LinkType::RAW,
            required: 1,
            actual: 0,
        })?;

        let layer = match first >> 4 {
            4 => LinkLayer::raw_ipv4(bytes),
            6 => LinkLayer::raw_ipv6(bytes),
            version => {
                return Err(LinkLayerError::InvalidIpVersion {
                    link_type: LinkType::RAW,
                    version,
                }
                .into());
            }
        };

        Ok(DecodedLink::new(layer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NetworkProtocol;

    #[test]
    fn version_nibble_selects_ipv4_or_ipv6_without_copying() {
        for (bytes, expected_protocol, expected_version) in [
            ([0x4f, 1], NetworkProtocol::Ipv4, 4),
            ([0x6f, 2], NetworkProtocol::Ipv6, 6),
        ] {
            let decoded = RawIpDecoder::decode(&bytes).expect("recognized RAW IP version");
            let (layer, protocol, payload) = decoded.into_parts();
            let raw = layer.as_raw_ip().expect("RAW view");

            assert_eq!(layer.link_type(), LinkType::RAW);
            assert_eq!(protocol, expected_protocol);
            assert_eq!(payload, bytes.as_slice());
            assert_eq!(payload.as_ptr(), bytes.as_ptr());
            assert_eq!(raw.ip_version, expected_version);
            assert_eq!(raw.payload.as_ptr(), bytes.as_ptr());
            assert!(layer.as_ethernet().is_none());
            assert!(layer.as_ieee80211().is_none());
        }
    }

    #[test]
    fn empty_raw_packet_is_a_link_layer_truncation() {
        assert!(matches!(
            RawIpDecoder::decode(&[]),
            Err(ParseError::InvalidLinkLayer(LinkLayerError::Truncated {
                link_type: LinkType::RAW,
                required: 1,
                actual: 0,
            }))
        ));
    }

    #[test]
    fn every_other_version_nibble_is_malformed() {
        for version in 0..=15 {
            if matches!(version, 4 | 6) {
                continue;
            }

            assert!(matches!(
                RawIpDecoder::decode(&[version << 4]),
                Err(ParseError::InvalidLinkLayer(
                    LinkLayerError::InvalidIpVersion {
                        link_type: LinkType::RAW,
                        version: actual,
                    }
                )) if actual == version
            ));
        }
    }
}
