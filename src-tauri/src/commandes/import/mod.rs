use log::{error, info};
use packet_parser::PacketFlow;
use pcap::Capture;
use std::{
    fs,
    io::ErrorKind,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError, label::LabelError},
    events::CaptureEvent,
    setup::labels::{clean_csv_field, parse_label_row},
    state::{
        capture::capture_handle::messages::capture::PacketMinimal, flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};

type ConflictsList = Vec<(String, String, String, String, String)>;

fn count_packets_in_pcap(file_path: &str) -> Result<usize, CaptureStateError> {
    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut count: usize = 0;
    while cap.next_packet().is_ok() {
        count += 1;
    }
    Ok(count)
}

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    app: tauri::AppHandle,
    pcap_paths: Vec<String>,
    on_event: Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    info!(
        "[convert_from_pcap_list] COMMAND CALLED avec pcap_paths = {:?}",
        pcap_paths
    );

    // started
    if let Err(e) = on_event.send(CaptureEvent::Started {
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 65536,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    };

    let mut matrice_guard = matrice.lock().unwrap();
    let mut graph_guard = graph.lock().unwrap();
    matrice_guard.clear();
    graph_guard.clear();

    labels_to_matrix(app.clone(), &mut matrice_guard)?;

    info!("[convert_from_pcap_list] Matrice & GraphData reset");

    for pcap_path in &pcap_paths {
        info!("[convert_from_pcap_list] Traitement de {}", pcap_path);
        handle_pcap_file(pcap_path, &mut matrice_guard, &mut graph_guard, &on_event)?;
    }

    info!("[convert_from_pcap_list] FIN traitement liste PCAP");

    // 🔥 snapshot complet envoyé sur le channel
    let snapshot: GraphData = graph_guard.get_all_graph_data(); // doit renvoyer un GraphData possédé

    if let Err(e) = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    }) {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }

    Ok(())
}

fn handle_pcap_file(
    file_path: &str,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    let total = count_packets_in_pcap(file_path)?;
    info!(
        "[handle_pcap_file] {} : {} paquets détectés",
        file_path, total
    );

    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut packet_count: usize = 0;

    while let Ok(packet) = cap.next_packet() {
        packet_count += 1;

        if let Ok(flow) = PacketFlow::try_from(packet.data) {
            let packet_min = PacketMinimal {
                ts_sec: packet.header.ts.tv_sec,
                ts_usec: packet.header.ts.tv_usec,
                caplen: packet.header.caplen,
                len: packet.header.len,
                flow,
            };

            let owned_packet = packet_min.to_owned_packet();
            matrice.update_flow(&owned_packet);
            let matrix_count = matrice.matrix.len();
            let source_ip = owned_packet
                .flow
                .internet
                .as_ref()
                .and_then(|i| i.source_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();
            let destination_ip = owned_packet
                .flow
                .internet
                .as_ref()
                .and_then(|i| i.destination_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();
            let source_label =
                matrice.get_label(&owned_packet.flow.data_link.source_mac, &source_ip);
            let destination_label = matrice.get_label(
                &owned_packet.flow.data_link.destination_mac,
                &destination_ip,
            );
            // info!(
            //     "[handle_pcap_file] {} : paquet {}/{} ; lignes matrice = {}",
            //     file_path,
            //     packet_count,
            //     total,
            //     matrix_count
            // );

            graph.add_packet_flow(&owned_packet.flow, source_label, destination_label);

            // Stats périodiques (optionnel)
            if (packet_count.is_multiple_of(1000) || packet_count == total)
                && let Err(e) = on_event.send(CaptureEvent::Stats {
                    received: packet_count as u32,
                    dropped: 0,
                    if_dropped: 0,
                    processed: matrix_count as u32,
                })
            {
                error!("Erreur lors de l'envoi de Stats: {:?}", e);
            }

            // si tu veux envoyer des Packet individuellement (live)
            // if let Err(e) = on_event.send(CaptureEvent::Packet { packet: &packet_min }) { ... }
        }
    }

    if let Err(e) = on_event.send(CaptureEvent::Finished {
        file_name: file_path,
        packet_total_count: total,
        matrix_total_count: matrice.matrix.len(),
    }) {
        error!("Erreur lors de l'envoi de Finished: {:?}", e);
    };
    info!(
        "[handle_pcap_file] Finised with {} paquets lu, {} lignes matrice",
        total,
        matrice.matrix.len()
    );
    Ok(())
}

/*<----- Csv part -----> */

#[tauri::command(async)]
pub fn get_label_files_list(app: tauri::AppHandle) -> Result<Option<String>, CaptureStateError> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");

    let count = fs::read_dir(&labels_folder)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .count();

    if count > 1 {
        return Err(LabelError::TooManyFiles { files_count: count }.into());
    }

    if fs::exists(&labels_folder).unwrap_or(false) {
        let file: Option<String> = fs::read_dir(&labels_folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("csv"))
            .filter_map(|path| path.file_name()?.to_str().map(String::from))
            .next();

        return Ok(file);
    }

    Ok(None)
}

#[tauri::command(async)]
pub fn remove_label_file(csv_file: String, app: AppHandle) -> Result<(), CaptureStateError> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");
    let label_file = labels_folder.join(&csv_file);

    fs::remove_file(label_file)?;
    println!("File removed");
    Ok(())
}

fn verif_file_format(file: String) -> Result<(), CaptureStateError> {
    let mut invalid_lines: Vec<String> = Vec::new();

    let file = match std::fs::read_to_string(&file) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    for line in file.lines() {
        let parts: Vec<_> = line.split(',').map(clean_csv_field).collect();
        if let [_unknown1, _unknown2] = parts.as_slice() {
            invalid_lines.push(line.to_string())
        }
    }

    if !invalid_lines.is_empty() {
        Err(LabelError::InvalidFileFormat { invalid_lines }.into())
    } else {
        Ok(())
    }
}

fn verif_labels_conflicts(file_path: String) -> Result<(), CaptureStateError> {
    let file: PathBuf = PathBuf::from(file_path);
    let file_name = file
        .file_name()
        .and_then(|f| f.to_str())
        .map(String::from)
        .unwrap_or(String::from("Nom du fichier inconnu"));

    let file = match std::fs::read_to_string(&file) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    let rows: Vec<(String, String, String)> = file.lines().filter_map(parse_label_row).collect();

    let new_file_with_name = (file_name, rows.clone());

    let mut same_ip_different_mac: ConflictsList = Vec::new();
    let mut same_ip_different_label: ConflictsList = Vec::new();

    for (i, (mac1, ip1, label1)) in rows.iter().enumerate() {
        for (mac2, ip2, label2) in rows[i + 1..].iter() {
            if ip1 == ip2 {
                if mac1 != mac2 {
                    eprintln!(
                        "⚠️  IP '{}' : MAC '{}' ({}) vs '{}' ({})",
                        ip1, mac1, new_file_with_name.0, mac2, new_file_with_name.0
                    );
                    same_ip_different_mac.push((
                        ip1.to_string(),
                        mac1.to_string(),
                        new_file_with_name.0.to_string(),
                        mac2.to_string(),
                        new_file_with_name.0.to_string(),
                    ))
                }

                if label1 != label2 {
                    eprintln!(
                        "⚠️  IP '{}' : label '{}' ({}) vs '{}' ({})",
                        ip1, label1, new_file_with_name.0, label2, new_file_with_name.0
                    );
                    same_ip_different_label.push((
                        ip1.to_string(),
                        label1.to_string(),
                        new_file_with_name.0.to_string(),
                        label2.to_string(),
                        new_file_with_name.0.to_string(),
                    ))
                }
            }
        }
    }

    if !same_ip_different_label.is_empty() || !same_ip_different_mac.is_empty() {
        Err(LabelError::LabelLinesConflicts {
            same_ip_diff_mac: same_ip_different_mac,
            same_ip_diff_label: same_ip_different_label,
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn verif_mac_ip_format(csv_path: String) -> Result<(), CaptureStateError> {
    let mut invalid_ip: Vec<(String, String)> = Vec::new();
    let mut invalid_mac: Vec<(String, String)> = Vec::new();
    println!("verif_mac_ip_format called with csv_path: {:?}", csv_path);

    let file = match std::fs::read_to_string(&csv_path) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    let rows: Vec<(String, String, String)> = file.lines().filter_map(parse_label_row).collect();
    println!("rows in verif_mac_ip_format: {:?}", rows);

    let name = Path::new(&csv_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("inconnu")
        .to_string();

    for (mac, ip, _label) in rows {
        if !is_ip_address(&ip) && !ip.is_empty() {
            invalid_ip.push((name.clone(), ip.to_string()));
        }
        if !is_mac_address(&mac) && !mac.is_empty() {
            invalid_mac.push((name.clone(), mac.to_string()));
        }
    }

    if !invalid_ip.is_empty() || !invalid_mac.is_empty() {
        return Err(LabelError::InvalidMacIpFormat {
            invalid_mac,
            invalid_ip,
        }
        .into());
    }
    Ok(())
}

fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

fn is_mac_address(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

#[tauri::command(async)]
pub fn import_label_files(csv_path: String, app: AppHandle) -> Result<(), CaptureStateError> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");

    verif_file_format(csv_path.clone())?;
    verif_mac_ip_format(csv_path.clone())?;
    verif_labels_conflicts(csv_path.clone())?;

    if fs::exists(&labels_folder).unwrap_or(false) {
        fs::remove_dir_all(&labels_folder)?;
    }
    fs::create_dir(&labels_folder)?;

    let dest = labels_folder.join(Path::new(&csv_path).file_name().unwrap());
    fs::copy(&csv_path, &dest)?;
    println!("copie de {:?} effectuée", csv_path);

    Ok(())
}

pub fn labels_to_matrix(
    app: tauri::AppHandle,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");

    let count = fs::read_dir(&labels_folder)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .count();

    if count > 1 {
        return Err(LabelError::TooManyFiles { files_count: count }.into());
    }

    if fs::exists(&labels_folder).unwrap_or(false) {
        let label_file: PathBuf = fs::read_dir(&labels_folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("csv"))
            .collect();

        let file = match std::fs::read_to_string(label_file) {
            Ok(csv_data) => csv_data,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };
        let labels: Vec<String> = file.lines().map(|l| l.to_string()).collect();

        for label in labels {
            let Some((mac, ip, label)) = parse_label_row(&label) else {
                continue;
            };

            matrice.add_label(mac.to_string(), ip, label);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn is_matrix_empty(state: tauri::State<'_, Arc<Mutex<FlowMatrix>>>) -> bool {
    state.lock().unwrap().matrix.is_empty()
}
