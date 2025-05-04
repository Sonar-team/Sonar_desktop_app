use serde::Serialize;
use crate::utils::return_device_lookup;

#[derive(Clone, Serialize)]
pub struct CaptureConfig {
    pub device_name: String,
    pub buffer_size: i32,
    pub timeout: i32,
}

impl CaptureConfig {
    pub fn default() -> Self {
        let device_name = return_device_lookup();
        Self {
            device_name,
            buffer_size: 18_000_000,
            timeout: 10_000,
        }
    }
    pub fn setup(&mut self, device_name: String, buffer_size: i32, timeout: i32) {
        self.device_name = device_name;
        self.buffer_size = buffer_size;
        self.timeout = timeout;
    }
    pub fn get_config(&self) -> (String, i32, i32) {
        (self.device_name.clone(), self.buffer_size, self.timeout)
    }
}