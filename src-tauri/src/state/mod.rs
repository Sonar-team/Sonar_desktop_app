//! État partagé de l'application (géré par Tauri, un `Arc<Mutex<…>>` par
//! domaine) : capture en cours, matrice de flux, graphe réseau et store de
//! labels.

pub mod capture;
pub mod flow_matrix;
pub mod graph;
pub mod labels_list;
