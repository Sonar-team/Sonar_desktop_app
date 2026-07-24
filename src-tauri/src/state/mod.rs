//! État partagé de l'application (géré par Tauri, un `Arc<Mutex<…>>` par
//! domaine) : capture en cours, matrice de flux, graphe réseau et store de
//! labels.
//!
//! # Ordre global des verrous (#166)
//!
//! `CaptureState` n'est jamais conservé pendant une jointure de thread ni
//! pendant le verrouillage d'un autre état. Lorsqu'une transaction doit
//! verrouiller plusieurs états de données, l'ordre unique est :
//! `FlowMatrix` → `GraphData` → `LabelStore` → `PcInfoLabel`, puis les
//! stores auxiliaires. Les notifications terminales du pipeline sont émises
//! après libération des verrous et normalisation de la phase.

pub mod capture;
pub mod flow_matrix;
pub mod graph;
pub mod labels_list;
