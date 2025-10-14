use packet_parser::owned::PacketFlowOwned;
use packet_parser::PacketFlow;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal<'a> {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlow<'a>,
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct PacketOwnedStats {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowOwned,
}

impl<'a> PacketMinimal<'a> {
    pub fn to_owned_packet(&self) -> PacketOwnedStats {
        PacketOwnedStats {
            ts_sec: self.ts_sec,
            ts_usec: self.ts_usec,
            caplen: self.caplen,
            len: self.len,
            flow: self.flow.to_owned(),
        }
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
