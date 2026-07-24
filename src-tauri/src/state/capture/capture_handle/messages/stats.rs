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
/// pool de buffers épuisé, canal capture→processing plein, ou paquet accepté
/// abandonné après une erreur fatale d'intégrité. Incrémentés par les workers,
/// lus par le thread de processing pour les stats.
#[derive(Debug, Default)]
pub struct AppDropCounters {
    pub no_buffer: AtomicU64,
    pub channel_full: AtomicU64,
    /// Paquets déjà acceptés dans le canal mais abandonnés après une erreur
    /// d'intégrité : retenter matrice/graphe serait incorrect, mais ils ne
    /// doivent pas disparaître du récapitulatif final (#158, #166).
    processing_fatal: AtomicU64,
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

    #[inline]
    pub fn add_processing_fatal(&self, count: u64) {
        self.processing_fatal.fetch_add(count, Ordering::Relaxed);
    }

    /// Total des pertes applicatives (pool + canal + arrêt fatal).
    #[inline]
    pub fn total(&self) -> u64 {
        self.no_buffer.load(Ordering::Relaxed)
            + self.channel_full.load(Ordering::Relaxed)
            + self.processing_fatal.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatTriple {
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub app_dropped: u64,
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

/// Photographie complète des compteurs de capture à un instant donné : stats
/// pcap + pertes applicatives ([`StatTriple`]) et erreurs de parsing. C'est
/// l'unité de déduplication des événements `Stats`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatsSnapshot {
    pub triple: StatTriple,
    /// Paquets acceptés par le pipeline mais illisibles par le parseur.
    pub parse_errors: u64,
    /// Flux distincts de la matrice (lignes du relevé).
    pub processed: u32,
}

// Miroir TypeScript généré sous le nom `Stats` : c'est la forme exacte des
// données de l'événement `stats` (#142).
#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "Stats", rename_all = "camelCase")]
pub struct StatsPayload {
    pub session_id: u64,
    pub received: u32,
    pub dropped: u32,
    pub if_dropped: u32,
    pub app_dropped: u64,
    pub parse_errors: u64,
    pub processed: u32,
}

impl StatsPayload {
    #[inline]
    pub fn new(snapshot: StatsSnapshot, session_id: u64) -> Self {
        Self {
            session_id,
            received: snapshot.triple.received,
            dropped: snapshot.triple.dropped,
            if_dropped: snapshot.triple.if_dropped,
            app_dropped: snapshot.triple.app_dropped,
            parse_errors: snapshot.parse_errors,
            processed: snapshot.processed,
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
            parse_errors: self.parse_errors,
            processed: self.processed,
        })
    }

    /// N'envoie que si le snapshot diffère du dernier émis. Tous les
    /// compteurs participent à la déduplication : une matrice qui grandit
    /// sans nouvelle perte (#154) comme un paquet illisible de plus doivent
    /// réémettre.
    #[inline]
    pub fn maybe_send(
        last: &mut Option<StatsSnapshot>,
        current: StatsSnapshot,
        session_id: u64,
        ch: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), tauri::Error> {
        if last.as_ref() == Some(&current) {
            return Ok(());
        }
        *last = Some(current);
        StatsPayload::new(current, session_id).send(ch)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn counting_channel() -> (Channel<CaptureEvent<'static>>, Arc<AtomicUsize>) {
        let sent = Arc::new(AtomicUsize::new(0));
        let counter = sent.clone();
        let ch = Channel::new(move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });
        (ch, sent)
    }

    /// Tous les compteurs participent à la déduplication : un snapshot
    /// identique n'est pas réémis, mais un paquet illisible ou intégré de
    /// plus doit réémettre — sinon le récapitulatif serait invisible (#158).
    #[test]
    fn maybe_send_deduplicates_on_the_full_snapshot() {
        let (ch, sent) = counting_channel();
        let mut last = None;
        let mut snapshot = StatsSnapshot {
            triple: StatTriple {
                received: 10,
                ..StatTriple::default()
            },
            parse_errors: 0,
            processed: 4,
        };

        StatsPayload::maybe_send(&mut last, snapshot, 1, &ch).unwrap();
        StatsPayload::maybe_send(&mut last, snapshot, 1, &ch).unwrap();
        assert_eq!(
            sent.load(Ordering::Relaxed),
            1,
            "snapshot identique dédupliqué"
        );

        snapshot.parse_errors += 1;
        StatsPayload::maybe_send(&mut last, snapshot, 1, &ch).unwrap();
        assert_eq!(
            sent.load(Ordering::Relaxed),
            2,
            "un paquet illisible de plus doit réémettre"
        );
    }
}
