use tauri::{AppHandle, Emitter};
use log::error;

/// Envoie un événement Tauri de manière générique, avec gestion d’erreur centralisée.
/// `event_name` : nom de l’événement à émettre.
/// `payload` : référence vers la donnée à envoyer (doit être sérialisable).
pub fn emit_event<T: serde::Serialize>(
    app: &AppHandle,
    event_name: &str,
    payload: &T,
) {
    if let Err(e) = app.emit_to("main", event_name, payload) {
        error!("[TAURI] Échec de l'émission '{}': {}", event_name, e);
    }
}