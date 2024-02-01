use tauri::State;
use log::error;

use crate::tauri_state::SonarState;

/// getting [(PacketInfos 
/// { 
/// mac_address_source: "2c:fd:a1:60:a1:83", 
/// mac_address_destination: "0a:87:c7:e6:6f:64", 
/// interface: "wlp6s0", 
/// l_3_protocol: "Ipv6", 
/// layer_3_infos: Layer3Infos { 
/// ip_source: Some("2a04:cec0:110d:e229:a32f:7cde:b5e0:76f"), 
/// ip_destination: Some("2606:4700:4400::ac40:90d4"), 
/// l_4_protocol: Some("Tcp"), layer_4_infos: Layer4Infos { 
/// port_source: Some("24590"), 
/// port_destination: Some("27013") } } }, 
/// 3),

pub fn get_graph_data(shared_vec_infopackets: State<SonarState>) -> Result<String, String> {
    // Attempt to acquire the lock on the shared state
    match shared_vec_infopackets.0.lock() {
        Ok(matrice) => {
            // Serialize the hash map to a JSON string
            print!("matrice: {:?}", matrice);
            serde_json::to_string(&*matrice).map_err(|e| {
                let err_msg = format!("Serialization error: {}", e);
                error!("{}", err_msg);
                err_msg
            })
        },
        Err(_) => {
            let err_msg = "Failed to lock the mutex".to_string();
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}