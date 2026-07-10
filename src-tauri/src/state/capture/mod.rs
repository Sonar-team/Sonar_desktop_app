//! État de la capture réseau : configuration, statut, handle des threads en
//! cours et channel d'événements vers le frontend.

use capture_config::CaptureConfig;
use capture_handle::CaptureHandle;
use capture_status::CaptureStatus;

use crate::events::CaptureEvent;
use tauri::ipc::Channel;

pub mod capture_config;
pub mod capture_handle;
pub mod capture_status;

/// État global d'une session de capture, partagé via `Arc<Mutex<…>>`.
pub struct CaptureState {
    /// Threads et canaux de la capture en cours (`None` à l'arrêt).
    pub capture: Option<CaptureHandle>,
    /// Statut courant (démarrée/arrêtée, interface, compteurs).
    pub status: CaptureStatus,
    /// Configuration appliquée au prochain démarrage.
    pub config: CaptureConfig,
    /// Filtre BPF actif, s'il y en a un.
    pub filter: Option<String>,
    /// Channel d'événements de la capture live : les commandes d'import s'en
    /// servent aussi pour joindre le front pendant une capture.
    pub on_event: Option<Channel<CaptureEvent<'static>>>,
}

impl CaptureState {
    /// État initial : aucune capture, statut et configuration par défaut.
    pub fn new() -> Self {
        Self {
            capture: None,
            status: CaptureStatus::default(),
            config: CaptureConfig::default(),
            filter: None,
            on_event: None,
        }
    }
}
