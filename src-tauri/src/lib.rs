pub mod capture_packet;
pub mod get_interfaces;
pub mod save_packets;
pub mod tauri_state;

use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    thread::{self, sleep},
    time::Duration,
};
use capture_packet::{all_interfaces, one_interface};
use clap::Parser;
use colored::Colorize;
use csv::Writer;


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

pub fn scan_for_time(app: tauri::AppHandle, output: &str, interface: &str, time: u64) -> Result<(), Box<dyn Error>> {
    println!(
        "Scanning {} interface(s) for {} seconds...",
        interface, time
    );
    let interface_clone = interface.to_owned();
    thread::spawn(move || {
        interfaces_handler(app, &interface_clone);
    });

    compte_a_rebours(time);
    create_csv(output)
}

fn compte_a_rebours(mut time: u64) {
    loop {
        println!(
            "{}",
            format!("Compte à rebours: {} secondes restantes", time).red()
        );
        if time == 0 {
            break;
        }
        time -= 1;
        sleep(Duration::from_secs(1));
    }
    println!("{}", "Compte à rebours: Temps écoulé!".red());
}

pub fn create_csv(output: &str) -> Result<(), Box<dyn Error>> {
    // creat a csv file
    let mut writer = Writer::from_path(output)?;
    // Fermez le fichier CSV (c'est important pour garantir que les données sont écrites)
    writer.flush()?;
    Ok(())
}

pub fn scan_until_interrupt(app: tauri::AppHandle, output: &str, interface: &str) -> Result<(), Box<dyn Error>> {
    interfaces_handler(app,interface);

    create_csv(output)
}

pub fn handle_interrupt(
    r: Arc<AtomicBool>,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Ctrl+C pressed. Exiting...");
    r.store(false, SeqCst);
    create_csv(output)
}

fn interfaces_handler(app: tauri::AppHandle,interface: &str) {
    match check_interface(interface) {
        true => all_interfaces(app),
        false => one_interface(app,interface),
    }
}

fn check_interface(interface: &str) -> bool {
    matches!(interface, "all")
}


mod tests_unitaires;
