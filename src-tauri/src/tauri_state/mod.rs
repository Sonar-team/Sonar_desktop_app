use std::{sync::{Mutex, Arc}, collections::HashSet};
use crate::capture_packet::layer_2_infos::PacketInfos;

pub struct SonarState {
    is_running: Mutex<bool>,
    pub matrice: Arc<Mutex<HashSet<PacketInfos>>>
}