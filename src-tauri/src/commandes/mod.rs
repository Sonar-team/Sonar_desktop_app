use tauri::command;


use whoami::fallible;

#[command]
pub fn get_hostname() -> String {
    fallible::hostname().unwrap()
}