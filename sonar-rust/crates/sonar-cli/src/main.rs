use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use sonar_core::{matrix::MatrixMergeRequest, pcap::PcapConvertRequest};

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
            PcapConvertRequest::new(inputs, output).map(|request| {
                eprintln!(
                    "pcap conversion is not implemented yet: {} input(s) -> {}",
                    request.inputs.len(),
                    request.output.display()
                );
            })
        }
        Command::Matrix { inputs, output } => {
            MatrixMergeRequest::new(inputs, output).map(|request| {
                eprintln!(
                    "matrix merge is not implemented yet: {} input(s) -> {}",
                    request.inputs.len(),
                    request.output.display()
                );
            })
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
