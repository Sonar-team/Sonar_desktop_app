use serde::Serialize;
use tauri::ipc::Channel;

use crate::events::CaptureEvent;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ChannelCapacityPayload {
    pub channel_size: usize,
    pub current_size: usize,
    pub backpressure: bool,
}

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
    pub fn send_if_changed(
        last: &mut Self,
        current_size: usize,
        max_size: usize,
        on_event: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), tauri::Error> {
        let backpressure = current_size >= (max_size as f32 * 0.9).floor() as usize;

        let current = Self {
            channel_size: max_size,
            current_size,
            backpressure,
        };

        if current != *last {
            *last = current.clone();
            on_event.send(CaptureEvent::ChannelCapacityPayload {
                channel_size: current.channel_size,
                current_size: current.current_size,
                backpressure: current.backpressure,
            })?;
        }

        if backpressure {
            log::warn!(
                "[BACKPRESSURE] Canal rempli à {}/{} ({}%)",
                current_size,
                max_size,
                (current_size * 100) / max_size
            );
        }

        Ok(())
    }
}
