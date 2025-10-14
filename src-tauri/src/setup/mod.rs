use colored::Colorize;
use log::info;

pub mod system_info;
pub fn get_os() {
    let platform = tauri_plugin_os::platform();
    info!("Platform: {}", platform);
}

pub fn print_banner() -> String {
    // ASCII art banner
    let banner = r"
    _________                           
   /   _____/ ____   ____ _____ _______ 
   \_____  \ /  _ \ /    \\__  \\_  __ \
   /        (  <_> )   |  \/ __ \|  | \/
  /_______  /\____/|___|  (____  /__|   
          \/            \/     \/          
   ";

    // La bannière est colorée en vert avant d'être retournée.
    banner.green().to_string()
}
