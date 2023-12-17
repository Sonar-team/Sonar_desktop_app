use std::sync::{Mutex, Arc};
use crate::capture_packet::layer_2_infos::PacketInfos;

pub struct SonarState (pub Arc<Mutex<Vec<PacketInfos>>>);

