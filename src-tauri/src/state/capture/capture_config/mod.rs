use crate::utils::return_device_lookup;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CaptureConfig {
    pub device_name: String,
    pub buffer_size: i32,
    pub chan_capacity: i32,
    pub timeout: i32,
    pub snaplen: i32,
}

impl CaptureConfig {
    pub fn default() -> Self {
        let device_name = return_device_lookup();
        Self {
            device_name,
            buffer_size: 18_000_000,
            chan_capacity: 10_000,
            timeout: 25,
            snaplen: 65536,
        }
    }
    pub fn setup(
        &mut self,
        device_name: String,
        buffer_size: i32,
        chan_capacity: i32,
        timeout: i32,
        snaplen: i32,
    ) {
        self.device_name = device_name;
        self.buffer_size = buffer_size;
        self.chan_capacity = chan_capacity;
        self.timeout = timeout;
        self.snaplen = snaplen;
    }
    pub fn get_config(&self) -> (String, i32, i32, i32, i32) {
        (
            self.device_name.clone(),
            self.buffer_size,
            self.chan_capacity,
            self.timeout,
            self.snaplen,
        )
    }
}
