use std::sync::{Arc, Mutex};

use pcap::Stat;

use crate::tauri_state::capture::capture_handle::threads::packet_buffer::PacketBuffer;

pub mod stats;
pub mod capture;
pub mod channel;

pub enum CaptureMessage {
    Packet(Arc<Mutex<PacketBuffer>>),
    Stats(Stat),
}