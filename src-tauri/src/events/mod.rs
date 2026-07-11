//! Événements poussés au frontend sur le `Channel` Tauri, pendant une
//! capture live comme pendant un import. Le contrat (noms `camelCase`,
//! forme `{ event, data }`) est consommé par `src/store/capture.ts`.

use serde::Serialize;

use crate::state::{
    capture::capture_handle::messages::capture::{PacketMinimal, PacketOwnedStats},
    graph::{GraphData, GraphUpdate},
};

/// Événement de capture/import envoyé au frontend. Les variantes empruntent
/// des références quand l'événement est construit depuis l'état verrouillé
/// (zéro copie à la sérialisation).
///
/// `session_id` identifie la session de capture live émettrice : le frontend
/// ignore les événements d'une session périmée. La valeur 0 signifie « hors
/// session » (imports PCAP/CSV), jamais filtrée.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum CaptureEvent<'a> {
    /// La capture (ou la conversion PCAP) démarre, avec ses paramètres.
    /// En capture live, émis seulement une fois l'interface ouverte et le
    /// filtre appliqué : un échec de démarrage ne produit jamais `Started`.
    Started {
        session_id: u64,
        device: &'a str,
        buffer_size: i32,
        chan_capacity: i32,
        timeout: i32,
        snaplen: i32,
    },
    /// Compteurs périodiques pour la barre de statut.
    Stats {
        session_id: u64,
        received: u32,
        dropped: u32,
        if_dropped: u32,
        /// Pertes côté application (pool de buffers épuisé ou canal plein),
        /// en plus des drops kernel remontés par pcap.
        app_dropped: u64,
        processed: u32,
    },
    /// Occupation du canal capture→processing (indicateur de backpressure).
    ChannelCapacityPayload {
        session_id: u64,
        channel_size: usize,
        current_size: usize,
        backpressure: bool,
    },
    /// Émission mono-paquet désactivée au profit de `PacketBatch` ; le
    /// variant reste dans le contrat d'événements du frontend
    /// (`src/store/capture.ts`) pour pouvoir être réactivé.
    #[allow(dead_code)]
    Packet { packet: &'a PacketMinimal<'a> },
    /// Lot de paquets traités (voir `PACKET_BATCH_MAX` côté processing).
    PacketBatch {
        session_id: u64,
        packets: Vec<PacketOwnedStats>,
    },
    /// Update graphe unitaire (ex. label de nœud modifié après arbitrage).
    Graph { update: &'a GraphUpdate },
    /// Updates graphe coalescées sur la fenêtre de batch (voir `GraphUpdateBatch`).
    GraphBatch {
        session_id: u64,
        updates: Vec<GraphUpdate>,
    },
    /// Fin du pipeline de capture, normale ou sur erreur fatale (ex. pcap).
    /// Sans cet événement, le frontend croirait capturer indéfiniment après
    /// une erreur.
    Stopped { session_id: u64, reason: String },
    /// Fin de traitement d'un fichier importé (PCAP ou matrice CSV), avec la
    /// comptabilité affichée dans la barre de statut.
    Finished {
        file_name: &'a str,
        packet_total_count: usize,
        matrix_total_count: usize,
    },
    /// Graphe complet, envoyé en fin d'import pour recharger la vue.
    GraphSnapshot { graph_data: &'a GraphData },
}
