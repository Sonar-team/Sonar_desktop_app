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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// (channel_size, current_size, backpressure) capturés à chaque envoi.
    type CapturedPayloads = Arc<Mutex<Vec<(usize, usize, bool)>>>;

    /// `Channel` de test : capture chaque payload `ChannelCapacityPayload`
    /// envoyé, sans dépendre d'un contexte Tauri réel.
    fn recording_channel() -> (Channel<CaptureEvent<'static>>, CapturedPayloads) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let captured = sent.clone();
        let on_event = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                let value: serde_json::Value = serde_json::from_str(&json).unwrap();
                if value["event"] == "channelCapacityPayload" {
                    let data = &value["data"];
                    captured.lock().unwrap().push((
                        data["channelSize"].as_u64().unwrap() as usize,
                        data["currentSize"].as_u64().unwrap() as usize,
                        data["backpressure"].as_bool().unwrap(),
                    ));
                }
            }
            Ok(())
        });
        (on_event, sent)
    }

    #[test]
    fn raises_backpressure_at_90_percent_fill() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 90, 100, 1, &on_event).unwrap();

        assert!(last.backpressure);
        assert_eq!(sent.lock().unwrap().as_slice(), &[(100, 90, true)]);
    }

    #[test]
    fn stays_clear_just_under_the_raise_threshold() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 89, 100, 1, &on_event).unwrap();

        assert!(!last.backpressure);
        assert_eq!(sent.lock().unwrap().as_slice(), &[(100, 89, false)]);
    }

    /// Hystérésis (#141) : une fois levé, le drapeau ne retombe qu'en
    /// dessous du seuil de relâchement (70 %), pas simplement sous 90 %.
    #[test]
    fn releases_only_below_the_release_threshold_not_the_raise_threshold() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, _sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 95, 100, 1, &on_event).unwrap();
        assert!(last.backpressure);

        // Sous 90 % mais toujours au-dessus de 70 % : le drapeau reste levé.
        ChannelCapacityPayload::send_if_changed(&mut last, 80, 100, 1, &on_event).unwrap();
        assert!(last.backpressure, "doit rester levé entre 70 % et 90 %");

        // Sous 70 % : relâché.
        ChannelCapacityPayload::send_if_changed(&mut last, 65, 100, 1, &on_event).unwrap();
        assert!(!last.backpressure);
    }

    #[test]
    fn no_event_sent_when_nothing_changed() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 10, 100, 1, &on_event).unwrap();
        ChannelCapacityPayload::send_if_changed(&mut last, 10, 100, 1, &on_event).unwrap();

        // Un seul événement : le second appel est un no-op (déduplication).
        assert_eq!(sent.lock().unwrap().len(), 1);
    }

    #[test]
    fn event_sent_on_first_call_from_default() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 0, 100, 1, &on_event).unwrap();

        assert_eq!(
            sent.lock().unwrap().len(),
            1,
            "l'état par défaut diffère de (0, 100, false)"
        );
    }

    #[test]
    fn payload_fields_round_trip_through_the_event() {
        let mut last = ChannelCapacityPayload::default();
        let (on_event, sent) = recording_channel();

        ChannelCapacityPayload::send_if_changed(&mut last, 42, 200, 7, &on_event).unwrap();

        assert_eq!(sent.lock().unwrap().as_slice(), &[(200, 42, false)]);
        assert_eq!(last.channel_size, 200);
        assert_eq!(last.current_size, 42);
    }
}
