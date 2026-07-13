// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use super::{DecodedLink, LinkDecoder};
use crate::{DataLink, LinkLayer, ParseError};

/// Decoder for Ethernet II frames, including the existing 802.1Q VLAN path.
pub(super) struct EthernetDecoder;

impl LinkDecoder for EthernetDecoder {
    #[inline(always)]
    fn decode<'a>(bytes: &'a [u8]) -> Result<DecodedLink<'a>, ParseError> {
        let frame = DataLink::try_from(bytes)?;
        Ok(DecodedLink::new(LinkLayer::ethernet(frame)))
    }
}
