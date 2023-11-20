use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use tauri::command;

fn save_to_csv(data: Vec<MyData>) -> Result<(), Box<dyn Error>> {
    // Ouvrir un fichier en écriture
    let file = File::create("output.csv")?;

    let mut wtr = csv::Writer::from_writer(file);

    // Écrire les données dans le fichier CSV
    for record in data {
        wtr.serialize(record)?;
    }

    // S'assurer que toutes les données sont écrites
    wtr.flush()?;

    Ok(())
}