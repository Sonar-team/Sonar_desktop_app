use pcap::Stat;
use pcap::{Packet, PacketCodec, PacketHeader};
use serde::Serialize;

#[cfg(target_os = "linux")]
use crate::tauri_state::capture::capture_handle::layer_2_infos::PacketInfos;

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
pub struct PacketFlow {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketInfos,
    pub formatted_time: String,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketFlow {
    pub ts_sec: i32,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketInfos,
    pub formatted_time: String,
    }

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize)]
pub struct PacketFlow {
    pub ts_sec: i64,
    pub ts_usec: i32,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketInfos,
    pub formatted_time: String,
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

use chrono::{NaiveDateTime, Timelike};

#[cfg(target_os = "linux")]
pub fn format_timestamp(ts_sec: i64, ts_usec: i64) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(ts_sec, (ts_usec * 1000) as u32)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());

    let micro = ts_usec % 1_000_000;

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        naive.hour(),
        naive.minute(),
        naive.second(),
        micro
    )
}

#[cfg(target_os = "windows")]
pub fn format_timestamp(ts_sec: i32, ts_usec: i32) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(ts_sec, (ts_usec * 1000) as u32)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());

    let micro = ts_usec % 1_000_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        naive.hour(),
        naive.minute(),
        naive.second(),
        micro
    )
}

#[cfg(target_os = "macos")]
pub fn format_timestamp(ts_sec: i64, ts_usec: i32) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(ts_sec, (ts_usec * 1000) as u32)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());

    let micro = ts_usec % 1_000_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        naive.hour(),
        naive.minute(),
        naive.second(),
        micro   
    )
}