use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatTriple {
    received: u32,
    dropped: u32,
    if_dropped: u32,
}

impl Default for StatTriple {
    fn default() -> Self {
        Self {
            received: u32::MAX,
            dropped: u32::MAX,
            if_dropped: u32::MAX,
        }
    }
}

impl From<pcap::Stat> for StatTriple {
    fn from(s: pcap::Stat) -> Self {
        Self {
            received: s.received,
            dropped: s.dropped,
            if_dropped: s.if_dropped,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct StatsPayload {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub processed: usize,
}

impl StatsPayload {
    pub fn from_stat_and_send(
        last: &mut StatTriple,
        stat: pcap::Stat,
        processed: usize,
        app: &AppHandle,
    ) -> Result<(), tauri::Error> {
        let current = StatTriple::from(stat);

        if *last != current {
            *last = current;
            let payload = Self {
                received: current.received,
                dropped: current.dropped,
                if_dropped: current.if_dropped,
                processed,
            };
            payload.send_new(app)
        } else {
            Ok(())
        }
    }

    pub fn send_new(&self, app: &AppHandle) -> Result<(), tauri::Error> {
        app.emit("stats", self)
    }
}

