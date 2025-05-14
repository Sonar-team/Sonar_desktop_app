use std::sync::{Arc, Mutex};
pub mod logs;

#[tauri::command(async, rename_all = "snake_case")]
pub fn save_packets_to_csv(
    file_path: String,
    state: State<'_, Arc<Mutex<SonarState>>>,
) -> Result<(), ExportError> {
    info!("Chemin d'enregistrement du CSV: {}", &file_path);
    let locked_state = state.lock().unwrap();
    locked_state.cmd_save_packets_to_csv(file_path)
}

#[command(async, rename_all = "snake_case")]
pub fn save_packets_to_excel(
    file_path: String,
    state: State<'_, Arc<Mutex<SonarState>>>,
) -> Result<(), ExportError> {
    info!("Chemin d'enregistrement du Excel: {}", &file_path);
    let locked_state = state.lock().unwrap();

    locked_state.cmd_save_packets_to_excel(file_path)
}

#[command(async)]
pub fn write_file(path: String, contents: String) -> Result<(), String> {
    info!("Chemin d'enregistrement du SVG: {}", &path);
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

use base64::{engine::general_purpose, Engine};
use log::{error, info};
use resvg::tiny_skia::Pixmap;
use tauri::{command, State};
use usvg::{Options, Transform, Tree};

use crate::{errors::export::ExportError, tauri_state::matrice::SonarState};

#[tauri::command(async)]
pub fn write_png_file(path: String, contents: String) -> Result<(), String> {
    // Journal pour vérifier le chemin et la taille des données
    info!("Chemin d'enregistrement du PNG : {}", &path);
    info!("Taille des données Base64 : {}", contents.len());

    // Décoder la chaîne Base64
    let decoded_data = match general_purpose::STANDARD.decode(contents) {
        Ok(data) => data,
        Err(e) => {
            error!("Erreur lors du décodage Base64 : {}", e);
            return Err(format!("Erreur lors du décodage Base64 : {}", e));
        }
    };

    // Écrire les données binaires dans le fichier
    match std::fs::write(&path, decoded_data) {
        Ok(_) => {
            info!("Fichier PNG écrit avec succès !");
            Ok(())
        }
        Err(e) => {
            error!("Erreur lors de l'écriture du fichier : {}", e);
            Err(format!("Erreur lors de l'écriture du fichier : {}", e))
        }
    }
}

#[tauri::command(async)]
pub fn write_file_as_png(path: String, contents: String) -> Result<(), String> {
    // Parse the SVG contents
    let opt = Options::default();
    let rtree = Tree::from_str(&contents, &opt).map_err(|e| e.to_string())?;

    // Create a pixmap with the dimensions of the SVG
    let pixmap_size = rtree.size();
    let mut pixmap = Pixmap::new(pixmap_size.width() as u32, pixmap_size.height() as u32)
        .ok_or("Failed to create pixmap")?;

    // Render the SVG onto the pixmap
    resvg::render(&rtree, Transform::identity(), &mut pixmap.as_mut());

    // Save the rendered image as a PNG file
    pixmap.save_png(&path).map_err(|e| e.to_string())?;

    Ok(())
}
