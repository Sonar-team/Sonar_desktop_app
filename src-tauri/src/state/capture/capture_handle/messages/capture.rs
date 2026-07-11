//! Représentations d'un paquet dans le pipeline : réexport du cœur partagé
//! `sonar-flows-core` (source unique du domaine, consommée aussi par
//! `sonar-cli`). `PacketMinimal::to_owned_packets` y déplie les tunnels en
//! niveaux de flux reliés par `encap_id`.

pub use sonar_flows_core::packet::*;
