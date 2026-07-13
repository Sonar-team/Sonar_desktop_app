// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Tunnel / encapsulation detection and peeling.
//!
//! Some packets carry a **whole other packet** inside their payload
//! (encapsulation). The base parser is layered and single-level, so without
//! help it only sees the *outer* flow and misses the real conversation nested
//! inside the tunnel.
//!
//! This module recognizes a tunnel from the transport layer, peels its
//! headers, exposes an honest inner IEEE 802.11 link layer and lets [`PacketFlow`] re-parse the
//! encapsulated packet recursively. One wire packet then yields several flow
//! levels (outer tunnel + inner conversation(s)).
//!
//! Currently supported:
//! - **CAPWAP-Data** (RFC 5415) carrying **IEEE 802.11** → **LLC/SNAP** → L3.
//!
//! Designed to grow: IP/UDP tunnels (VXLAN, GRE, GTP-U, IP-in-IP) plug into
//! [`detect_inner`] the same way.

use super::PacketFlow;
use super::data_link::ethertype::Ethertype;
use super::data_link::mac_addres::MacAddress;
use super::link::DecodedLink;
use super::link_layer::{Ieee80211Link, LinkLayer};
use super::transport::Transport;
use super::transport::protocols::TransportProtocol;

/// Maximum tunnel nesting depth (anti-loop guard against malformed traffic that
/// could claim endless encapsulation). The outer flow is depth 0.
pub(crate) const MAX_TUNNEL_DEPTH: u8 = 4;

/// UDP port of the CAPWAP data plane (RFC 5415).
const CAPWAP_DATA_PORT: u16 = 5247;

/// Detects an encapsulation on the transport layer. On success returns
/// `(tunnel_name, inner_flow)`: the name is meant for the *outer* flow's
/// application-protocol field, and `inner_flow` is the fully re-parsed
/// encapsulated packet.
///
/// Returns `None` (graceful degradation, never an error) when there is no
/// tunnel, the payload is encrypted (e.g. CAPWAP/DTLS), truncated, or uses a
/// shape we don't decode yet.
pub(crate) fn detect_inner<'a>(
    transport: &Transport<'a>,
    depth: u8,
) -> Option<(&'static str, PacketFlow<'a>)> {
    if depth + 1 >= MAX_TUNNEL_DEPTH {
        return None;
    }
    let payload = transport.payload?;

    // --- CAPWAP-Data over UDP 5247 → 802.11 → LLC/SNAP → L3 ---
    if transport.protocol == TransportProtocol::Udp
        && (transport.source_port == Some(CAPWAP_DATA_PORT)
            || transport.destination_port == Some(CAPWAP_DATA_PORT))
        && let Some(inner_link) = peel_capwap_ieee80211(payload)
        && let Ok(inner) = PacketFlow::parse_decoded(DecodedLink::new(inner_link), depth + 1)
    {
        return Some(("CAPWAP", inner));
    }

    None
}

/// Peels CAPWAP-Data → IEEE 802.11 → LLC/SNAP and returns the inner data-link
/// layer (802.11 MAC addresses + SNAP EtherType + L3 payload).
fn peel_capwap_ieee80211(payload: &[u8]) -> Option<LinkLayer<'_>> {
    // --- CAPWAP header (RFC 5415) ---
    // Byte 0: preamble = version(4 bits) | type(4 bits). Type 0 = plaintext,
    // type 1 = DTLS (encrypted) → we can't recurse into it.
    if payload.len() < 8 || payload[0] & 0x0f != 0 {
        return None;
    }
    // HLEN (5 bits, top of byte 1) = header length in 4-byte words.
    let capwap_header = ((payload[1] >> 3) & 0x1f) as usize * 4;
    if capwap_header < 8 || payload.len() < capwap_header {
        return None;
    }

    peel_ieee80211(&payload[capwap_header..])
}

/// Peels an IEEE 802.11 **data** frame and its LLC/SNAP header into an inner
/// data-link layer. Handles ToDS/FromDS addressing, the optional Address4
/// (WDS) and the optional QoS control field.
fn peel_ieee80211(frame: &[u8]) -> Option<LinkLayer<'_>> {
    if frame.len() < 24 {
        return None;
    }
    // Frame Control is 2 octets: one carries version/type/subtype, the other the
    // flags. Cisco CAPWAP captures sometimes byte-swap them (Wireshark shows
    // "(Swapped)"). The version bits (low 2 bits of the type octet) are 0 for
    // real frames, so we use that to tell which octet is which.
    let (fc_type, fc_flags) = if frame[0] & 0x03 == 0 {
        (frame[0], frame[1])
    } else if frame[1] & 0x03 == 0 {
        (frame[1], frame[0])
    } else {
        return None;
    };

    // Only data frames (type 2) carry an upper-layer payload we can recurse on.
    if (fc_type >> 2) & 0x03 != 2 {
        return None;
    }
    let subtype = (fc_type >> 4) & 0x0f;
    let to_ds = fc_flags & 0x01 != 0;
    let from_ds = fc_flags & 0x02 != 0;

    // Header length: base 24 (+6 for Address4 in WDS, +2 for QoS control).
    let mut header = 24usize;
    if to_ds && from_ds {
        header += 6;
    }
    if subtype & 0x08 != 0 {
        header += 2; // QoS data subtypes (>= 8)
    }
    if frame.len() < header {
        return None;
    }

    // Real source/destination depend on ToDS/FromDS (802.11 address mapping).
    let a1 = &frame[4..10];
    let a2 = &frame[10..16];
    let a3 = &frame[16..22];
    let (dst, src): (&[u8], &[u8]) = match (to_ds, from_ds) {
        (false, false) => (a1, a2),           // IBSS: DA=A1, SA=A2
        (false, true) => (a1, a3),            // from AP: DA=A1, SA=A3
        (true, false) => (a3, a2),            // to AP: DA=A3, SA=A2
        (true, true) => (a3, &frame[24..30]), // WDS: DA=A3, SA=A4
    };

    let (ethertype, l3) = peel_llc_snap(&frame[header..])?;

    Some(LinkLayer::ieee80211(Ieee80211Link::new(
        MacAddress(dst.try_into().ok()?),
        MacAddress(src.try_into().ok()?),
        Ethertype(ethertype),
        l3,
    )))
}

/// Peels an LLC/SNAP header (DSAP=SSAP=0xAA, control=0x03, OUI=00:00:00) and
/// returns the encapsulated EtherType and the remaining L3 payload. Only the
/// SNAP form (which carries an EtherType) is handled.
fn peel_llc_snap(llc: &[u8]) -> Option<(u16, &[u8])> {
    if llc.len() < 8 {
        return None;
    }
    if llc[0] != 0xAA || llc[1] != 0xAA || llc[2] != 0x03 {
        return None;
    }
    if llc[3] != 0x00 || llc[4] != 0x00 || llc[5] != 0x00 {
        return None;
    }
    let ethertype = u16::from_be_bytes([llc[6], llc[7]]);
    Some((ethertype, &llc[8..]))
}
