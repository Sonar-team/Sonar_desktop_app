//! Matrice de flux : réexport du cœur partagé `sonar-flows-core`, source
//! unique du domaine (consommée aussi par `sonar-cli`). Le desktop n'est
//! qu'un adaptateur : commandes Tauri → fonctions du cœur → événements UI.

pub use sonar_flows_core::matrix::*;
