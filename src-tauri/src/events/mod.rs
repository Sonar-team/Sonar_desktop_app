use serde::Serialize;

use crate::state::{
    capture::capture_handle::messages::capture::{PacketMinimal, PacketOwnedStats},
    graph::{GraphData, GraphUpdate},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum CaptureEvent<'a> {
    Started {
        device: &'a str,
        buffer_size: i32,
        chan_capacity: i32,
        timeout: i32,
        snaplen: i32,
    },
    Stats {
        received: u32,
        dropped: u32,
        if_dropped: u32,
        /// Pertes côté application (pool de buffers épuisé ou canal plein),
        /// en plus des drops kernel remontés par pcap.
        app_dropped: u64,
        processed: u32,
    },
    ChannelCapacityPayload {
        channel_size: usize,
        current_size: usize,
        backpressure: bool,
    },
    Packet {
        packet: &'a PacketMinimal<'a>,
    },
    PacketBatch {
        packets: Vec<PacketOwnedStats>,
    },
    Graph {
        update: &'a GraphUpdate,
    },
    /// Updates graphe coalescées sur la fenêtre de batch (voir `GraphUpdateBatch`).
    GraphBatch {
        updates: Vec<GraphUpdate>,
    },
    /// Fin du pipeline de capture, normale ou sur erreur fatale (ex. pcap).
    /// Sans cet événement, le frontend croirait capturer indéfiniment après
    /// une erreur.
    Stopped {
        reason: String,
    },
    Finished {
        file_name: &'a str,
        packet_total_count: usize,
        matrix_total_count: usize,
    },
    GraphSnapshot {
        graph_data: &'a GraphData,
    },
}
