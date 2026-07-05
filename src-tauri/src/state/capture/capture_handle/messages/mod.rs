use crate::state::capture::capture_handle::threads::packet_buffer::PacketBuffer;

pub mod capture;
pub mod channel;
pub mod stats;

/// Le canal capture→processing ne transporte que des paquets : les stats
/// passent par [`stats::SharedCaptureStats`] pour ne jamais être perdues
/// quand le canal est plein.
pub enum CaptureMessage {
    Packet(PacketBuffer),
}
