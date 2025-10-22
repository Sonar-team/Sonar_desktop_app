use pcap::Stat;

use crate::state::capture::capture_handle::threads::packet_buffer::PacketBuffer;

pub mod capture;
pub mod channel;
pub mod stats;
pub enum CaptureMessage {
    Packet(PacketBuffer),
    Stats(Stat),
}
