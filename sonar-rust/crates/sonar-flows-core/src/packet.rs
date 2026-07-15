//! Représentations d'un paquet dans le pipeline : [`CapturedPacket`]
//! (emprunte les octets capturés) et [`CapturedPacketOwned`] (possédé,
//! sérialisable vers le frontend), avec dépliage des tunnels en niveaux de
//! flux reliés par `encap_id`.

use std::net::IpAddr;

use packet_parser::PacketFlow;
use packet_parser::owned::PacketFlowOwned;

use serde::Serialize;

use crate::link::LinkView;

/// FNV-1a 64 bits sur une sérialisation d'octets contrôlée : contrairement à
/// `DefaultHasher`, dont la stabilité n'est pas garantie entre versions de
/// Rust, deux builds différents produisent les mêmes `encap_id` — les
/// matrices exportées restent joignables entre elles (#148).
fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    })
}

/// Sérialisation préfixe-libre des champs d'une extrémité de tunnel.
fn push_endpoint(buf: &mut Vec<u8>, mac: &str, ip: Option<IpAddr>, port: Option<u16>) {
    buf.extend_from_slice(&(mac.len() as u32).to_be_bytes());
    buf.extend_from_slice(mac.as_bytes());
    match ip {
        None => buf.push(0),
        Some(IpAddr::V4(v4)) => {
            buf.push(4);
            buf.extend_from_slice(&v4.octets());
        }
        Some(IpAddr::V6(v6)) => {
            buf.push(6);
            buf.extend_from_slice(&v6.octets());
        }
    }
    match port {
        None => buf.push(0),
        Some(p) => {
            buf.push(1);
            buf.extend_from_slice(&p.to_be_bytes());
        }
    }
}

fn push_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u32).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

/// Identifiant déterministe d'une paire de tunnel, utilisé comme `encap_id`
/// pour relier la ligne de tunnel externe à ses lignes internes. Les deux
/// sens d'un même tunnel (A -> B et B -> A) partagent le même identifiant :
/// les extrémités sont ordonnées avant hachage. Le hachage (FNV-1a sur une
/// sérialisation explicite) est stable entre builds et versions de Rust.
pub fn tunnel_pair_id(flow: &PacketFlowOwned) -> u64 {
    let link = LinkView::of(&flow.data_link);
    let source = (
        &link.source_mac,
        flow.internet.as_ref().and_then(|i| i.source_ip),
        flow.transport.as_ref().and_then(|t| t.source_port),
    );
    let destination = (
        &link.destination_mac,
        flow.internet.as_ref().and_then(|i| i.destination_ip),
        flow.transport.as_ref().and_then(|t| t.destination_port),
    );
    let (first, second) = if source <= destination {
        (source, destination)
    } else {
        (destination, source)
    };

    let mut buf = Vec::with_capacity(128);
    push_endpoint(&mut buf, first.0, first.1, first.2);
    push_endpoint(&mut buf, second.0, second.1, second.2);
    push_str(&mut buf, &link.protocol);
    // Seul l'id du VLAN participe : c'est le seul champ préservé par un
    // aller-retour CSV (pcp/dei non exportés).
    match link.vlan_id {
        None => buf.push(0),
        Some(id) => {
            buf.push(1);
            buf.extend_from_slice(&id.to_be_bytes());
        }
    }
    push_str(&mut buf, flow.internet.as_ref().map_or("", |i| &i.protocol));
    push_str(
        &mut buf,
        flow.transport.as_ref().map_or("", |t| &t.protocol),
    );
    push_str(
        &mut buf,
        flow.application.as_ref().map_or("", |a| &a.protocol),
    );
    fnv1a(&buf)
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacket<'a> {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacket<'a> {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacket<'a> {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

/// Même convention hex que `Edge::encap_ids` (`graph.rs`) et la colonne CSV
/// `encap_id` (`matrix.rs`) : un hash FNV-1a 64 bits perd en précision une
/// fois passé tel quel par un `number` JSON (`Number.MAX_SAFE_INTEGER` <
/// `u64::MAX`), donc jamais sérialisé comme entier brut.
fn serialize_encap_id_hex<S: serde::Serializer>(
    value: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(id) => serializer.serialize_str(&format!("{id:016x}")),
        None => serializer.serialize_none(),
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacketOwned {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé. Sérialisé en hex
    /// 16 caractères (même convention que `Edge::encap_ids` et la colonne
    /// CSV `encap_id`) : c'est un hash FNV-1a 64 bits qui peut dépasser
    /// `Number.MAX_SAFE_INTEGER`, imprécis une fois passé par un `number`
    /// JSON côté frontend (#142).
    #[serde(serialize_with = "serialize_encap_id_hex")]
    pub encap_id: Option<u64>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacketOwned {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé. Sérialisé en hex
    /// 16 caractères (même convention que `Edge::encap_ids` et la colonne
    /// CSV `encap_id`) : c'est un hash FNV-1a 64 bits qui peut dépasser
    /// `Number.MAX_SAFE_INTEGER`, imprécis une fois passé par un `number`
    /// JSON côté frontend (#142).
    #[serde(serialize_with = "serialize_encap_id_hex")]
    pub encap_id: Option<u64>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPacketOwned {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
    /// Identifiant du tunnel encapsulant (partagé par la ligne externe et ses
    /// lignes internes). `None` pour un flux non tunnelé. Sérialisé en hex
    /// 16 caractères (même convention que `Edge::encap_ids` et la colonne
    /// CSV `encap_id`) : c'est un hash FNV-1a 64 bits qui peut dépasser
    /// `Number.MAX_SAFE_INTEGER`, imprécis une fois passé par un `number`
    /// JSON côté frontend (#142).
    #[serde(serialize_with = "serialize_encap_id_hex")]
    pub encap_id: Option<u64>,
}

impl<'a> CapturedPacket<'a> {
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

    pub fn to_owned_packet(&self) -> CapturedPacketOwned {
        let mut flow = self.flow.to_owned();
        // La ligne externe d'un tunnel doit porter le même `encap_id` que ses
        // lignes internes (celui calculé dans `to_owned_packets`), sinon la
        // jointure externe <-> interne est impossible côté SOC.
        let encap_id = self.flow.inner.is_some().then(|| tunnel_pair_id(&flow));
        // Le flux encapsulé ne fait PAS partie de l'identité de la ligne
        // externe : il est matérialisé comme ligne interne à part entière,
        // jointe par `encap_id`. Le garder dans la clé faisait éclater un
        // même tunnel en une ligne externe par conversation interne — une
        // distinction que l'export CSV ne sait pas exprimer, donc perdue au
        // réimport (aller-retour non inversible, vu sur ndpi_capwap.pcap).
        flow.inner = None;
        CapturedPacketOwned {
            ts_sec: self.ts_sec,
            ts_usec: self.ts_usec,
            caplen: self.caplen,
            len: self.len,
            flow,
            encap_id,
        }
    }

    /// Convertit le paquet et tous ses niveaux encapsulés (tunnels) en une liste
    /// de `CapturedPacketOwned`, du plus externe au plus interne. Un paquet non
    /// tunnelé donne un seul élément (comportement identique à `to_owned_packet`).
    ///
    /// La taille en octets est attribuée **par niveau** — trame complète pour
    /// l'externe, taille du segment L3 pour chaque niveau interne — afin de ne
    /// pas compter deux fois le même volume.
    pub fn to_owned_packets(&self) -> Vec<CapturedPacketOwned> {
        let levels = self.flow.flatten();
        let tunneled = levels.len() > 1;

        let mut owned: Vec<CapturedPacketOwned> = levels
            .into_iter()
            .enumerate()
            .map(|(depth, flow)| {
                let len = if depth == 0 {
                    self.len
                } else {
                    flow.data_link.network_payload().len() as u32
                };
                // Même règle que `to_owned_packet` : chaque niveau est une
                // ligne à part entière, le flux qu'il encapsule n'entre pas
                // dans son identité (jointure par `encap_id`).
                let mut owned_flow = flow.to_owned();
                owned_flow.inner = None;
                CapturedPacketOwned {
                    ts_sec: self.ts_sec,
                    ts_usec: self.ts_usec,
                    caplen: self.caplen,
                    len,
                    flow: owned_flow,
                    encap_id: None,
                }
            })
            .collect();

        // Pour un paquet tunnelé, la ligne externe et ses lignes internes
        // partagent le même identifiant : le hash du flux le plus externe.
        // Le SOC peut ainsi joindre externe <-> interne (GROUP BY encap_id).
        if tunneled {
            let id = tunnel_pair_id(&owned[0].flow);
            for packet in &mut owned {
                packet.encap_id = Some(id);
            }
        }

        owned
    }
}

// impl <'a> CapturedPacket<'a> {
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
// Le cfg composé échappe à `allow-unwrap-in-tests` (clippy.toml) : on
// réautorise explicitement les unwrap dans ce module de tests.
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        let packet = CapturedPacket {
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

    /// Les deux sens d'un même tunnel doivent porter le même `encap_id`,
    /// sinon les fils de l'aller et du retour ne se joignent pas au même père.
    #[test]
    fn tunnel_pair_id_is_direction_agnostic() {
        let bytes = decode_hex(
            "c464138f9e04442b0302172c080045080138b7e04000ff115967ac18086aac1808ca2174147f0124000000200320000000000104e5440000000001082c00003a9a5af450e0c2642fa3b4000c29967ca43980aaaa030000000800453800eca89440008006651464ac911e64ac91b4dd7b01bda58ecb952483ab67501800fec87e0000000000c0fe534d4240000100030000000500000030000000000000006704090000000000fffe0000966e611ec3ba49eb000000000000000000000000000000000000000039000000020000000000000000000000000000000000000080000000000000000700000001000000000020007800120090000000300000005200530058005f00440052002d0053004600000000000000180000001000040000001800000000004d78416300000000000000001000040000001800000000005146696400000000",
        );
        let flow = PacketFlow::try_from(bytes.as_slice()).expect("parse CAPWAP frame");
        let forward = flow.to_owned();

        // Retour : mêmes extrémités, sens inversé.
        let mut reverse = forward.clone();
        let mut swapped = reverse
            .data_link
            .as_ethernet()
            .expect("trame Ethernet")
            .clone();
        std::mem::swap(&mut swapped.source_mac, &mut swapped.destination_mac);
        reverse.data_link = packet_parser::owned::LinkLayerOwned::ethernet(swapped);
        if let Some(internet) = reverse.internet.as_mut() {
            std::mem::swap(&mut internet.source_ip, &mut internet.destination_ip);
            std::mem::swap(
                &mut internet.ip_source_type,
                &mut internet.ip_destination_type,
            );
        }
        if let Some(transport) = reverse.transport.as_mut() {
            std::mem::swap(&mut transport.source_port, &mut transport.destination_port);
        }

        assert_ne!(forward, reverse, "les deux sens sont des flux distincts");
        assert_eq!(tunnel_pair_id(&forward), tunnel_pair_id(&reverse));
    }

    #[test]
    fn to_owned_packets_untunneled_has_no_encap_id() {
        // Ethernet -> IPv4 -> UDP simple (non tunnelé) : un seul niveau, encap_id None.
        let bytes = decode_hex(
            "00112233445566778899aabb08004500001c000100004011000ac0a80001c0a800020035003500080000",
        );
        let flow = PacketFlow::try_from(bytes.as_slice()).expect("parse UDP frame");
        let packet = CapturedPacket {
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
