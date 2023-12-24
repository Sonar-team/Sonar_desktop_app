pub mod capture_packet;
pub mod get_interfaces;
pub mod save_packets;
pub mod tauri_state;

use capture_packet::{all_interfaces, one_interface};
use clap::Parser;
use colored::Colorize;
use tauri_state::SonarState;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name the output name of the csv
    #[arg(short, long, default_value = "output.csv")]
    output: String,
    #[arg(short, long, default_value = "all")]
    /// give the interface name to scan
    interface: String,
    /// Give the scan time
    #[arg(short, long, default_value_t = 0)]
    time: u64,
}

pub fn get_args(args: &Args) -> (&String, &String, &u64) {
    (&args.output, &args.interface, &args.time)
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

    banner.green().to_string()
}

pub fn scan_until_interrupt(
    app: tauri::AppHandle, 
    interface: &str, 
    state: tauri::State<SonarState>)  
    {
        match check_interface(interface) {
            true => all_interfaces(app, state),
            false => one_interface(app,interface, state),
        }
    }

fn check_interface(interface: &str) -> bool {
    matches!(interface, "all")
}


mod tests_unitaires;
