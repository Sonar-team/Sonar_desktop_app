//! Statistiques de capture partagées entre threads (atomiques, hors canal de
//! données) et payload `Stats` émis au frontend.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use serde::Serialize;
use tauri::ipc::Channel;

use crate::events::CaptureEvent;

/// Dernières stats pcap connues, partagées entre le thread de capture (écrit)
/// et le thread de processing (lit sur timer). Hors du canal de données pour
/// rester fiables sous backpressure — un canal plein perdrait précisément les
/// stats au moment où elles sont les plus utiles.
#[derive(Debug, Default)]
pub struct SharedCaptureStats {
    received: AtomicU32,
    dropped: AtomicU32,
    if_dropped: AtomicU32,
}

impl SharedCaptureStats {
    pub fn store(&self, stat: pcap::Stat) {
        self.received.store(stat.received, Ordering::Relaxed);
        self.dropped.store(stat.dropped, Ordering::Relaxed);
        self.if_dropped.store(stat.if_dropped, Ordering::Relaxed);
    }

    /// Snapshot courant ; `app_dropped` est complété par l'appelant.
    pub fn load(&self) -> StatTriple {
        StatTriple {
            received: self.received.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            if_dropped: self.if_dropped.load(Ordering::Relaxed),
            app_dropped: 0,
        }
    }
}

/// Pertes de paquets côté application (en plus des drops kernel de pcap) :
/// pool de buffers épuisé ou canal capture→processing plein. Incrémentés par
/// le thread de capture, lus par le thread de processing pour les stats.
#[derive(Debug, Default)]
pub struct AppDropCounters {
    pub no_buffer: AtomicU64,
    pub channel_full: AtomicU64,
}

impl AppDropCounters {
    #[inline]
    pub fn add_no_buffer(&self) {
        self.no_buffer.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn add_channel_full(&self) {
        self.channel_full.fetch_add(1, Ordering::Relaxed);
    }

    /// Total des pertes applicatives (pool + canal).
    #[inline]
    pub fn total(&self) -> u64 {
        self.no_buffer.load(Ordering::Relaxed) + self.channel_full.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatTriple {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub app_dropped: u64,
}

impl Default for StatTriple {
    fn default() -> Self {
        Self {
            received: u32::MAX,
            dropped: u32::MAX,
            if_dropped: u32::MAX,
            app_dropped: u64::MAX,
        }
    }
}

impl From<pcap::Stat> for StatTriple {
    fn from(s: pcap::Stat) -> Self {
        Self {
            received: s.received,
            dropped: s.dropped,
            if_dropped: s.if_dropped,
            app_dropped: 0,
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
    pub session_id: u64,
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub app_dropped: u64,
    pub processed: u32,
}

impl StatsPayload {
    #[inline]
    pub fn new(triple: StatTriple, processed: u32, session_id: u64) -> Self {
        Self {
            session_id,
            received: triple.received,
            dropped: triple.dropped,
            if_dropped: triple.if_dropped,
            app_dropped: triple.app_dropped,
            processed,
        }
    }

    /// Envoie immédiatement le payload (aucune déduplication ici).
    #[inline]
    pub fn send(&self, ch: &Channel<CaptureEvent<'static>>) -> Result<(), tauri::Error> {
        ch.send(CaptureEvent::Stats {
            session_id: self.session_id,
            received: self.received,
            dropped: self.dropped,
            if_dropped: self.if_dropped,
            app_dropped: self.app_dropped,
            processed: self.processed,
        })
    }

    /// Compare avec `last`/`last_processed` et n’envoie que si changement.
    /// `processed` participe à la déduplication : sans lui, une matrice qui
    /// grandit sans nouvelle perte n'était jamais réémise (#154).
    #[inline]
    pub fn maybe_send(
        last: &mut StatTriple,
        last_processed: &mut u32,
        mut current: StatTriple,
        app_dropped: u64,
        processed: u32,
        session_id: u64,
        ch: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), tauri::Error> {
        current.app_dropped = app_dropped;
        let triple_changed = current.update_if_changed(last);
        let processed_changed = *last_processed != processed;
        *last_processed = processed;
        if triple_changed || processed_changed {
            let payload = StatsPayload::new(current, processed, session_id);
            payload.send(ch)
        } else {
            Ok(())
        }
    }
}
