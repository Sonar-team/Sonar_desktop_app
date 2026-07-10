//! Threads du pipeline de capture : lecture pcap ([`capture`]), traitement
//! matrice/graphe/IPC ([`processing`]), pool de buffers ([`packet_buffer`])
//! et instrumentation optionnelle (`capture_timing`).

pub mod capture;
#[cfg(feature = "capture_timing")]
mod capture_timing;
pub mod packet_buffer;
pub mod processing;
