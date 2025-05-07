use pcap::Stat;
use pcap::{Packet, PacketCodec, PacketHeader};
use serde::Serialize;

pub enum CaptureMessage {
    Packet(PacketOwned),
    Stats(Stat),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: PacketHeader,
    pub data: Box<[u8]>,
}

pub struct Codec;

impl PacketCodec for Codec {
    type Item = PacketOwned;

    fn decode(&mut self, packet: Packet) -> Self::Item {
        PacketOwned {
            header: *packet.header,
            data: packet.data.into(),
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub data: Vec<u8>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub data: Vec<u8>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketMinimal {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Serialize)]
pub struct StatsPayload {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub processed: u32,
}

#[derive(Clone, Serialize)]
pub struct ChannelCapacityPayload {
    pub channel_size: usize,
    pub current_size: usize,
    pub backpressure: bool,
}
