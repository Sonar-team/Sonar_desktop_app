use std::hash::{Hash, Hasher};

use packet_parser::PacketFlow;
use packet_parser::owned::PacketFlowOwned;

use serde::Serialize;

/// Identifiant déterministe d'un flux (hash de sa clé), utilisé comme
/// `encap_id` pour relier une ligne de tunnel externe à ses lignes internes.
fn flow_encap_id(flow: &PacketFlowOwned) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    flow.hash(&mut hasher);
    hasher.finish()
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal<'a> {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal<'a> {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal<'a> {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct PacketOwnedStats {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé.
    pub encap_id: Option<u64>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct PacketOwnedStats {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé.
    pub encap_id: Option<u64>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct PacketOwnedStats {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé.
    pub encap_id: Option<u64>,
}

impl<'a> PacketMinimal<'a> {
    // pub fn new(pkt: PacketBuffer) -> Result<Self, ParsedPacketError> {
    //     let flow = PacketFlow::try_from(pkt.data.as_ref())?;
    //     Ok(Self {
    //         ts_sec: pkt.header.ts.tv_sec,
    //         ts_usec: pkt.header.ts.tv_usec,
    //         caplen: pkt.header.caplen,
    //         len: pkt.header.len,
    //         flow,
    //     })
    // }

    pub fn to_owned_packet(&self) -> PacketOwnedStats {
        let flow = self.flow.to_owned();
        // La ligne externe d'un tunnel doit porter le même `encap_id` que ses
        // lignes internes (celui calculé dans `to_owned_packets`), sinon la
        // jointure externe <-> interne est impossible côté SOC.
        let encap_id = self.flow.inner.is_some().then(|| flow_encap_id(&flow));
        PacketOwnedStats {
            ts_sec: self.ts_sec,
            ts_usec: self.ts_usec,
            caplen: self.caplen,
            len: self.len,
            flow,
            encap_id,
        }
    }

    /// Convertit le paquet et tous ses niveaux encapsulés (tunnels) en une liste
    /// de `PacketOwnedStats`, du plus externe au plus interne. Un paquet non
    /// tunnelé donne un seul élément (comportement identique à `to_owned_packet`).
    ///
    /// La taille en octets est attribuée **par niveau** — trame complète pour
    /// l'externe, taille du segment L3 pour chaque niveau interne — afin de ne
    /// pas compter deux fois le même volume.
    pub fn to_owned_packets(&self) -> Vec<PacketOwnedStats> {
        let levels = self.flow.flatten();
        let tunneled = levels.len() > 1;

        let mut owned: Vec<PacketOwnedStats> = levels
            .into_iter()
            .enumerate()
            .map(|(depth, flow)| {
                let len = if depth == 0 {
                    self.len
                } else {
                    flow.data_link.payload.len() as u32
                };
                PacketOwnedStats {
                    ts_sec: self.ts_sec,
                    ts_usec: self.ts_usec,
                    caplen: self.caplen,
                    len,
                    flow: flow.to_owned(),
                    encap_id: None,
                }
            })
            .collect();

        // Pour un paquet tunnelé, la ligne externe et ses lignes internes
        // partagent le même identifiant : le hash du flux le plus externe.
        // Le SOC peut ainsi joindre externe <-> interne (GROUP BY encap_id).
        if tunneled {
            let id = flow_encap_id(&owned[0].flow);
            for packet in &mut owned {
                packet.encap_id = Some(id);
            }
        }

        owned
    }
}

// impl <'a> PacketMinimal<'a> {
//     pub fn new(pkt: PacketBuffer) -> Result<Self, ParsedPacketError> {
//         let flow = PacketFlow::try_from(pkt.data.as_ref())?;
//         Ok(Self {
//             ts_sec: pkt.header.ts.tv_sec,
//             ts_usec: pkt.header.ts.tv_usec as i32,
//             caplen: pkt.header.caplen,
//             len: pkt.header.len,
//             flow,
//         })
//     }
// }

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Trame CAPWAP → 802.11 → LLC/SNAP → IPv4 → TCP:445 (SMB2). Vérifie que le
    /// paquet tunnelé est aplati en deux niveaux de flux, avec l'attribution des
    /// octets par niveau (trame complète pour l'externe, segment L3 pour l'interne).
    #[test]
    fn to_owned_packets_splits_capwap_tunnel_into_two_levels() {
        let bytes = decode_hex(
            "c464138f9e04442b0302172c080045080138b7e04000ff115967ac18086aac1808ca2174147f0124000000200320000000000104e5440000000001082c00003a9a5af450e0c2642fa3b4000c29967ca43980aaaa030000000800453800eca89440008006651464ac911e64ac91b4dd7b01bda58ecb952483ab67501800fec87e0000000000c0fe534d4240000100030000000500000030000000000000006704090000000000fffe0000966e611ec3ba49eb000000000000000000000000000000000000000039000000020000000000000000000000000000000000000080000000000000000700000001000000000020007800120090000000300000005200530058005f00440052002d0053004600000000000000180000001000040000001800000000004d78416300000000000000001000040000001800000000005146696400000000",
        );
        let flow = PacketFlow::try_from(bytes.as_slice()).expect("parse CAPWAP frame");
        let packet = PacketMinimal {
            ts_sec: 1,
            ts_usec: 2,
            caplen: 326,
            len: 326,
            flow,
        };

        let levels = packet.to_owned_packets();
        assert_eq!(levels.len(), 2, "un paquet CAPWAP -> deux niveaux de flux");

        // Externe : trame complète (326 o) + protocole applicatif "CAPWAP".
        assert_eq!(levels[0].len, 326);
        assert_eq!(
            levels[0]
                .flow
                .application
                .as_ref()
                .map(|a| a.protocol.as_str()),
            Some("CAPWAP")
        );

        // Interne : conversation TCP:445, octets = taille du segment L3 interne (236 o).
        let inner_transport = levels[1].flow.transport.as_ref().expect("inner transport");
        assert_eq!(inner_transport.destination_port, Some(445));
        assert_eq!(levels[1].len, 236);

        // encap_id : présent et IDENTIQUE entre l'externe et l'interne (jointure SOC).
        assert!(levels[0].encap_id.is_some());
        assert_eq!(levels[0].encap_id, levels[1].encap_id);

        // Le pipeline construit la ligne externe via `to_owned_packet` : elle
        // doit porter le même encap_id que les niveaux issus de `to_owned_packets`.
        assert_eq!(packet.to_owned_packet().encap_id, levels[0].encap_id);
    }

    #[test]
    fn to_owned_packets_untunneled_has_no_encap_id() {
        // Ethernet -> IPv4 -> UDP simple (non tunnelé) : un seul niveau, encap_id None.
        let bytes = decode_hex(
            "00112233445566778899aabb08004500001c000100004011000ac0a80001c0a800020035003500080000",
        );
        let flow = PacketFlow::try_from(bytes.as_slice()).expect("parse UDP frame");
        let packet = PacketMinimal {
            ts_sec: 1,
            ts_usec: 2,
            caplen: bytes.len() as u32,
            len: bytes.len() as u32,
            flow,
        };

        let levels = packet.to_owned_packets();
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].encap_id, None);
        assert_eq!(packet.to_owned_packet().encap_id, None);
    }
}
