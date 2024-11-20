use whoami::fallible;

#[tauri::command(async, rename_all = "snake_case")]
pub fn get_hostname_to_string() -> String {
    fallible::hostname().unwrap()
}