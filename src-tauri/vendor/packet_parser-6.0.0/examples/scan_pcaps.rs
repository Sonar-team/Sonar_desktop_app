// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Balaye des fichiers pcap/pcapng et compte les protocoles applicatifs
//! détectés par `PacketFlow`, fichier par fichier.
//!
//! Usage :
//!   cargo run --example scan_pcaps -- <pcap|dossier>... [--focus PROTO]
//!
//! Avec `--focus`, chaque trame détectée comme PROTO est détaillée
//! (n° de trame, ports, premiers octets du payload) pour auditer les
//! faux positifs.

use packet_parser::PacketFlow;
use pcap::Capture;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn collect_pcaps(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<_> = match std::fs::read_dir(path) {
            Ok(rd) => rd.flatten().map(|e| e.path()).collect(),
            Err(_) => return,
        };
        entries.sort();
        for entry in entries {
            collect_pcaps(&entry, out);
        }
    } else if matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("pcap") | Some("pcapng") | Some("cap")
    ) {
        out.push(path.to_path_buf());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let focus = args
        .iter()
        .position(|a| a == "--focus")
        .map(|i| args.remove(i + 1))
        .inspect(|_| {
            args.retain(|a| a != "--focus");
        });

    if args.is_empty() {
        eprintln!("usage: scan_pcaps <pcap|dossier>... [--focus PROTO]");
        std::process::exit(2);
    }

    let mut files = Vec::new();
    for arg in &args {
        collect_pcaps(Path::new(arg), &mut files);
    }

    let mut grand_total: BTreeMap<String, usize> = BTreeMap::new();

    for file in &files {
        let mut cap = match Capture::from_file(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: illisible ({e})", file.display());
                continue;
            }
        };

        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut frame_no = 0usize;
        while let Ok(packet) = cap.next_packet() {
            frame_no += 1;
            let Ok(flow) = PacketFlow::try_from(packet.data) else {
                *tally.entry("<erreur L2>".into()).or_default() += 1;
                continue;
            };

            // La détection applicative pertinente est celle du flux le plus
            // interne (après dé-tunnellisation éventuelle).
            let flows = flow.flatten();
            let innermost = flows.last().expect("flatten n'est jamais vide");
            let label = match &innermost.application {
                Some(app) => app.application_protocol.to_string(),
                None => "<pas de L7>".to_string(),
            };

            if let Some(focus_proto) = &focus
                && label.eq_ignore_ascii_case(focus_proto)
            {
                let fmt_port = |p: Option<u16>| p.map_or("?".into(), |p| p.to_string());
                let (sport, dport, head) = match &innermost.transport {
                    Some(t) => (
                        fmt_port(t.source_port),
                        fmt_port(t.destination_port),
                        t.payload
                            .map(|p| {
                                p.iter()
                                    .take(16)
                                    .map(|b| format!("{b:02X}"))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            })
                            .unwrap_or_default(),
                    ),
                    None => ("?".into(), "?".into(), String::new()),
                };
                println!(
                    "{}: trame {frame_no} {sport}->{dport} [{head}]",
                    file.display()
                );
            }

            *tally.entry(label).or_default() += 1;
        }

        println!("\n== {} ({frame_no} trames)", file.display());
        for (proto, count) in &tally {
            println!("   {proto:<12} {count}");
            *grand_total.entry(proto.clone()).or_default() += count;
        }
    }

    println!("\n== TOTAL ({} fichiers)", files.len());
    for (proto, count) in &grand_total {
        println!("   {proto:<12} {count}");
    }

    Ok(())
}
