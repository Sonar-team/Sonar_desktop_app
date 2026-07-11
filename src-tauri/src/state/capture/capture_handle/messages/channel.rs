//! Indicateur d'occupation du canal capture→processing, remonté au frontend
//! pour matérialiser la backpressure.

use serde::Serialize;
use tauri::ipc::Channel;

use crate::events::CaptureEvent;

/// Occupation du canal : taille max, remplissage courant et drapeau de
/// backpressure (hystérésis : levé à ≥ 90 % de remplissage, relâché sous
/// 70 % — le drapeau ne clignote pas autour du seuil, #141).
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ChannelCapacityPayload {
    pub channel_size: usize,
    pub current_size: usize,
    pub backpressure: bool,
}

/// Seuil de levée du drapeau backpressure (fraction de remplissage).
const BACKPRESSURE_RAISE: f32 = 0.9;
/// Seuil de relâchement, plus bas que la levée (hystérésis).
const BACKPRESSURE_RELEASE: f32 = 0.7;

impl Default for ChannelCapacityPayload {
    fn default() -> Self {
        Self {
            channel_size: usize::MAX,
            current_size: usize::MAX,
            backpressure: false,
        }
    }
}

impl ChannelCapacityPayload {
    /// Émet l'événement `ChannelCapacityPayload` seulement si l'état a changé
    /// depuis `last` (déduplication côté backend).
    pub fn send_if_changed(
        last: &mut Self,
        current_size: usize,
        max_size: usize,
        session_id: u64,
        on_event: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), tauri::Error> {
        // Hystérésis : le seuil de relâchement est plus bas que celui de
        // levée, le drapeau ne clignote pas autour de 90 % (#141).
        let threshold = if last.backpressure {
            BACKPRESSURE_RELEASE
        } else {
            BACKPRESSURE_RAISE
        };
        let backpressure = current_size >= (max_size as f32 * threshold).floor() as usize;

        let current = Self {
            channel_size: max_size,
            current_size,
            backpressure,
        };

        // Log seulement aux transitions : sous saturation prolongée, un log
        // par émission noyait le fichier précisément quand le pipeline était
        // déjà sous pression (#141).
        if backpressure != last.backpressure {
            if backpressure {
                log::warn!(
                    "[BACKPRESSURE] Canal rempli à {}/{} ({}%)",
                    current_size,
                    max_size,
                    (current_size * 100) / max_size.max(1)
                );
            } else {
                log::info!(
                    "[BACKPRESSURE] Retour à la normale ({}/{})",
                    current_size,
                    max_size
                );
            }
        }

        if current != *last {
            *last = current.clone();
            on_event.send(CaptureEvent::ChannelCapacityPayload {
                session_id,
                channel_size: current.channel_size,
                current_size: current.current_size,
                backpressure: current.backpressure,
            })?;
        }

        Ok(())
    }
}
