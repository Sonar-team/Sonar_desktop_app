//! CLI d'analyse batch SONAR : conversion PCAP → matrice CSV et fusion de
//! matrices, sans interface graphique (mêmes traitements que l'application
//! desktop, via `sonar-flows-core`).

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

mod graphviz;
mod sigma;

use graphviz::{GraphEngine, GraphOptions, NodeLabels};

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

    /// Render one or more Sonar matrix files (.csv or .xlsx) as a relational graph.
    Graph {
        /// Sonar matrix input files (.csv or .xlsx).
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output path (.dot, .svg, .png, or .json for the sigma.js renderer).
        #[arg(short, long)]
        output: PathBuf,

        /// Ignore relations carrying fewer bytes.
        #[arg(long, default_value_t = 0)]
        min_bytes: u64,

        /// Keep at most this many nodes, ranked by incident traffic.
        #[arg(long, default_value_t = 200)]
        max_nodes: usize,

        /// Keep only this protocol (case-insensitive, for example TCP).
        #[arg(long)]
        protocol: Option<String>,

        /// Text displayed on nodes.
        #[arg(long, value_enum, default_value_t = NodeLabels::Ip)]
        labels: NodeLabels,

        /// Graphviz layout engine used for SVG and PNG output.
        #[arg(long, value_enum, default_value_t = GraphEngine::Sfdp)]
        engine: GraphEngine,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Pcap { inputs, output } => {
            sonar_flows_core::pcap::convert_pcap_files_to_csv(&inputs, &output, |path, report| {
                // Bilan par catégorie fine (#150) : le total « lus » est la
                // somme des catégories, l'équation tient par construction.
                eprintln!(
                    "{}: {} paquet(s) lus = {} intégré(s) + {} tronqué(s) \
                     + {} DLT non supporté(s) + {} malformé(s)",
                    path.display(),
                    report.read(),
                    report.decoded,
                    report.rejected_truncated,
                    report.rejected_unsupported_link_type,
                    report.rejected_malformed
                );
            })
            .map(|rows| (rows, output))
            .map_err(|error| error.to_string())
        }
        Command::Matrix { inputs, output } => {
            sonar_flows_core::csv::merge_matrix_files_to_csv(&inputs, &output)
                .map(|rows| (rows, output))
                .map_err(|error| error.to_string())
        }
        Command::Graph {
            inputs,
            output,
            min_bytes,
            max_nodes,
            protocol,
            labels,
            engine,
        } => sonar_flows_core::csv::merge_matrix_files(&inputs)
            .map_err(|error| error.to_string())
            .and_then(|matrix| {
                let graph = sonar_flows_core::graph::GraphData::from_flow_matrix_filtered(
                    &matrix,
                    protocol.as_deref(),
                );
                let options = GraphOptions {
                    min_bytes,
                    max_nodes,
                    labels,
                    engine,
                };
                // Une sortie `.json` alimente le rendu sigma.js
                // (`script/graph/render-sigma.mjs`), qui produit la même image
                // que le graphe de l'application. Les autres extensions
                // passent par Graphviz.
                let is_json = output
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
                if is_json {
                    sigma::export_graph_json(&graph, &output, &options)
                        .map(|relations| (relations, output))
                } else {
                    graphviz::export_graph(&graph, &output, &options)
                        .map(|relations| (relations, output))
                }
            }),
    };

    match result {
        Ok((rows, output)) => {
            eprintln!("{} élément(s) exporté(s) vers {}", rows, output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
