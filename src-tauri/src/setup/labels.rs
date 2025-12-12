use tauri::{AppHandle, Manager, path::BaseDirectory};

pub fn read_labels(app: &AppHandle) -> Result<(), tauri::Error> {
    let resource_path = app
        .path()
        .resolve("resources/labels.csv", BaseDirectory::Resource)?;
    println!("resource_path: {:?}", resource_path);
    // read in file and display :
    let csv_data = std::fs::read_to_string(resource_path.clone())?;
    println!("{}", csv_data);
    Ok(())
}
