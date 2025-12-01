use serde::Serialize;

use crate::state::{capture::capture_handle::messages::capture::PacketMinimal, graph::GraphUpdate};

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
    Graph {
        update: &'a GraphUpdate,
    },
    Finished {
        file_name: &'a str,
        packet_total_count: usize,
        matrix_total_count: usize,
    },
}
