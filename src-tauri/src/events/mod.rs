//! Événements poussés au frontend sur le `Channel` Tauri, pendant une
//! capture live comme pendant un import. Le contrat (noms `camelCase`,
//! forme `{ event, data }`) est consommé par `src/store/capture.ts`.
//!
//! ## Convention de nommage sérialisée (#142)
//!
//! `camelCase` partout — tags d'événement, tags de variante interne
//! (`GraphUpdate`) et champs de struct — sur tout ce qui est *Sonar-owned*
//! (ce module, `StatsPayload`, `CapturedPacketOwned`, `Node`/`Edge`/
//! `GraphUpdate` de `sonar_flows_core`). Seule exception assumée : la
//! couche paquet/flux (`PacketFlow`, `DataLink`, `IpType`, `NetworkProtocol`,
//! `CorruptedLayerKind`…) reste dans la casse imposée par la crate vendorée
//! `packet_parser` (`snake_case` pour les champs, `snake_case` ou
//! `PascalCase` selon l'enum pour les tags) — cette crate ne se modifie
//! jamais ici (cf. `never-edit-vendor-packet-parser`) ; à signaler en amont
//! si une uniformisation complète devient nécessaire.

use serde::Serialize;

// Miroir TypeScript de ce module (#142), utilisé uniquement à l'export du
// contrat IPC et par son test de fidélité JSON : jamais construit hors tests.
#[cfg(test)]
pub mod contract;

use crate::state::{
    capture::capture_handle::messages::capture::CapturedPacketOwned,
    graph::{GraphData, GraphUpdate},
};

/// Version du contrat `CaptureEvent` (#142) : à incrémenter à toute évolution
/// non rétro-compatible (variante retirée, champ renommé/retiré, forme JSON
/// changée) — jamais pour un ajout de champ optionnel ou de variante, qui
/// reste sûr à l'exécution : `src/store/capture.ts` journalise (au lieu de
/// planter) tout tag d'événement qu'il ne reconnaît pas encore. Portée par
/// `Started`, émis en tête de toute session (capture live, import PCAP et
/// import de matrice CSV), pour que le frontend la lise dès le début du
/// flux et signale un écart (`EXPECTED_PROTOCOL_VERSION` côté store).
pub const CAPTURE_EVENT_PROTOCOL_VERSION: u32 = 1;

/// Événement de capture/import envoyé au frontend. Les variantes empruntent
/// des références quand l'événement est construit depuis l'état verrouillé
/// (zéro copie à la sérialisation).
///
/// `session_id` identifie la session de capture live émettrice : le frontend
/// ignore les événements d'une session périmée. La valeur 0 signifie « hors
/// session » (imports PCAP/CSV), jamais filtrée.
#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
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
        /// Type de liaison (LINKTYPE/DLT) de la capture ou du fichier importé
        /// (« Ethernet », « Linux cooked v1 »…), affiché dans la barre de
        /// statut. Plusieurs fichiers importés de DLT différents sont joints
        /// par des virgules.
        link_type: &'a str,
        /// Version du contrat IPC, cf. [`CAPTURE_EVENT_PROTOCOL_VERSION`].
        protocol_version: u32,
    },
    /// Compteurs périodiques pour la barre de statut. Émis aussi une dernière
    /// fois après le drainage d'arrêt ou de plafond (#158) : le récapitulatif
    /// final où chaque paquet reçu appartient à une catégorie — intégré,
    /// illisible, ou perdu (noyau / interface / application).
    Stats {
        session_id: u64,
        received: u32,
        dropped: u32,
        if_dropped: u32,
        /// Pertes côté application (pool de buffers épuisé ou canal plein),
        /// en plus des drops kernel remontés par pcap.
        app_dropped: u64,
        /// Rejets du parseur par catégorie fine (#150) : capture coupée au
        /// snaplen, type de liaison inconnu du parser, trame malformée sur
        /// le réseau observé. Même partition que l'import de fichier.
        rejected_truncated: u64,
        rejected_unsupported_link_type: u64,
        rejected_malformed: u64,
        processed: u32,
    },
    /// Occupation du canal capture→processing (indicateur de backpressure).
    ChannelCapacityPayload {
        session_id: u64,
        channel_size: usize,
        current_size: usize,
        backpressure: bool,
    },
    /// Lot de paquets traités (voir `PACKET_BATCH_MAX` côté processing).
    PacketBatch {
        session_id: u64,
        packets: Vec<CapturedPacketOwned>,
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
    /// Fin de traitement d'un fichier importé (PCAP ou matrice CSV), avec le
    /// rapport qualité du fichier (#150) : chaque paquet lu appartient à
    /// exactement une catégorie fine, et le total émis est la somme des
    /// catégories (`PcapFileReport::read()`) — l'équation
    /// `lus = intégrés + tronqués + DLT non supportés + malformés`
    /// tient par construction. Affiché dans la barre de statut.
    Finished {
        file_name: &'a str,
        packet_total_count: usize,
        /// Paquets parsés et intégrés à la matrice (pour une matrice CSV :
        /// lignes importées).
        integrated_count: usize,
        /// Rejetés parce que la capture était coupée (`caplen < len`) :
        /// snaplen trop court au moment du relevé, pas un défaut du réseau.
        rejected_truncated_count: usize,
        /// Rejetés pour type de liaison inconnu du parser. Zéro attendu :
        /// la garde DLT rejette le fichier entier sinon — le bilan prouve
        /// ce zéro au lieu de le supposer.
        rejected_unsupported_link_type_count: usize,
        /// Rejetés trame complète : malformés sur le réseau observé (0 pour
        /// une matrice CSV : une ligne invalide est une erreur fatale, pas
        /// un skip).
        rejected_malformed_count: usize,
        matrix_total_count: usize,
    },
    /// Progression d'un import multi-fichiers. `current` et `total` comptent
    /// les paquets pour un PCAP et les lignes pour une matrice CSV ; les index
    /// de fichier commencent à 1 pour être affichés directement dans l'UI.
    ImportProgress {
        file_name: &'a str,
        file_index: usize,
        files_total: usize,
        current: usize,
        total: usize,
    },
    /// Graphe complet, envoyé en fin d'import pour recharger la vue.
    GraphSnapshot { graph_data: &'a GraphData },
}
