use serde::Serialize;
use tauri::ipc::Channel;

use crate::events::CaptureEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatTriple {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
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

impl StatTriple {
    /// Retourne true si la stat est différente de `last` et met `last` à jour.
    #[inline]
    pub fn update_if_changed(self, last: &mut StatTriple) -> bool {
        if *last != self {
            *last = self;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct StatsPayload {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub processed: u32,
}

impl StatsPayload {
    #[inline]
    pub fn new(triple: StatTriple, processed: u32) -> Self {
        Self {
            received: triple.received,
            dropped: triple.dropped,
            if_dropped: triple.if_dropped,
            processed,
        }
    }

    /// Envoie immédiatement le payload (aucune déduplication ici).
    #[inline]
    pub fn send(&self, ch: &Channel<CaptureEvent<'static>>) -> Result<(), tauri::Error> {
        // Variante A : event avec payload struct
        ch.send(CaptureEvent::Stats {
            received: self.received,
            dropped: self.dropped,
            if_dropped: self.if_dropped,
            processed: self.processed,
        })
        // Variante B si tu as `CaptureEvent::Stats { payload: StatsPayload }`
        // ch.send(CaptureEvent::Stats { payload: *self })
    }

    /// Compare avec `last` et n’envoie que si changement.
    #[inline]
    pub fn maybe_send(
        last: &mut StatTriple,
        stat: pcap::Stat,
        processed: u32,
        ch: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), tauri::Error> {
        let current = StatTriple::from(stat);
        if current.update_if_changed(last) {
            let payload = StatsPayload::new(current, processed);
            payload.send(ch)
        } else {
            Ok(())
        }
    }
}
