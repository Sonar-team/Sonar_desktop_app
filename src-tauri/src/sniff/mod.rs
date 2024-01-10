pub(crate) mod capture_packet;
use capture_packet::{all_interfaces, one_interface};

use crate::tauri_state::SonarState;

pub fn scan_until_interrupt(
    app: tauri::AppHandle,
    interface: &str,
    state: tauri::State<SonarState>,
) {
    match check_interface(interface) {
        true => all_interfaces(app, state),
        false => one_interface(app, interface, state),
    }
}

fn check_interface(interface: &str) -> bool {
    matches!(interface, "all")
}
