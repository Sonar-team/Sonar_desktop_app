//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use std::sync::{Arc, Mutex};

use crate::sniff::capture_packet::layer_2_infos::PacketInfos;

/// `SonarState` encapsule l'état global de l'application Sonar.
///
/// Cette structure est conçue pour stocker et gérer les informations sur les trames réseau
/// capturées, y compris le comptage de leurs occurrences.
///
/// # Structure
/// `SonarState` contient un `Arc<Mutex<Vec<(PacketInfos, u32)>>>`.
/// - `Arc` permet un accès thread-safe et partagé à l'état.
/// - `Mutex` garantit que l'accès à l'état est mutuellement exclusif,
///   empêchant les conditions de concurrence.
/// - `Vec<(PacketInfos, u32)>` stocke les trames réseau (`PacketInfos`) et
///   leur nombre d'occurrences (`u32`).
///
/// # Exemple
/// ```
/// use std::sync::{Mutex, Arc};
/// use std::collections::HashMap;
/// use crate::capture_packet::layer_2_infos::PacketInfos;
/// use crate::SonarState;
///
/// let state = SonarState(Arc::new(Mutex::new(Vec::new())));
/// // Utilisez `state` ici pour gérer les trames réseau et leur comptage
/// ```

pub struct SonarState(pub Arc<Mutex<Vec<(PacketInfos, u32)>>>);
