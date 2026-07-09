use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sonar-cli")]
#[command(about = "Sonar batch analysis CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Convert one or more PCAP files into a Sonar flow matrix CSV.
    Pcap {
        /// PCAP, PCAPNG, or CAP input files.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output matrix CSV path.
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Merge one or more Sonar matrix CSV files.
    Matrix {
        /// Sonar matrix CSV input files.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output merged matrix CSV path.
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Pcap { inputs, output } => {
            sonar_flows_core::pcap::convert_pcap_files_to_csv(&inputs, &output, |path, report| {
                eprintln!(
                    "{}: {} paquet(s) lus, {} intégré(s), {} non parsé(s)",
                    path.display(),
                    report.packets,
                    report.parse_ok,
                    report.parse_errors
                );
            })
            .map(|rows| (rows, output))
        }
        Command::Matrix { inputs, output } => {
            sonar_flows_core::csv::merge_matrix_files_to_csv(&inputs, &output).map(|rows| (rows, output))
        }
    };

    match result {
        Ok((rows, output)) => {
            eprintln!("{} flux exporté(s) vers {}", rows, output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
