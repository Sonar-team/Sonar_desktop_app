// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

use crate::LinkType;

/// Failure while identifying or decoding a canonical LINKTYPE payload.
///
/// Ethernet keeps its historical `DataLinkError` conversion for compatibility.
/// New link decoders use this link-aware contract so capture consumers can
/// account for truncation and malformed input without parsing error strings.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
pub enum LinkLayerError {
    #[error(
        "LINKTYPE {link_type} packet is truncated: required bytes {required}, actual bytes {actual}"
    )]
    Truncated {
        link_type: LinkType,
        required: usize,
        actual: usize,
    },

    #[error("Malformed LINKTYPE {link_type} packet: invalid IP version nibble {version}")]
    InvalidIpVersion { link_type: LinkType, version: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_error_keeps_link_type_and_lengths() {
        let error = LinkLayerError::Truncated {
            link_type: LinkType::RAW,
            required: 1,
            actual: 0,
        };

        assert_eq!(
            error.to_string(),
            "LINKTYPE 101 packet is truncated: required bytes 1, actual bytes 0"
        );
    }

    #[test]
    fn malformed_raw_error_keeps_invalid_version() {
        let error = LinkLayerError::InvalidIpVersion {
            link_type: LinkType::RAW,
            version: 7,
        };

        assert_eq!(
            error.to_string(),
            "Malformed LINKTYPE 101 packet: invalid IP version nibble 7"
        );
    }
}
