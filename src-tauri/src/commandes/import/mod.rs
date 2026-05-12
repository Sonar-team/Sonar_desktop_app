use log::{error, info};
use packet_parser::PacketFlow;
use pcap::Capture;
use std::{sync::{Arc, Mutex}, 
    io::ErrorKind, fs, 
    path::{PathBuf},
    collections::HashSet
};
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError}, events::CaptureEvent, setup::labels::parse_label_row, state::{
        capture::capture_handle::messages::capture::PacketMinimal, 
        flow_matrix::FlowMatrix,
        graph::GraphData, label_files_list::SelectedLabelFiles,
    }
};

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
    label : State<'_, Arc<Mutex<SelectedLabelFiles>>>,
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

    labels_to_matrix(app.clone(), &mut matrice_guard, label)?;

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
pub fn read_label_files_list(
    app: tauri::AppHandle,
    state : State<'_, Arc<Mutex<SelectedLabelFiles>>>
) -> Result<Vec<(String, bool)>, tauri::Error>{
    let s = state.lock().unwrap();
    let selected_label_files_names_list = s.get().clone();
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");
    let mut label_files_list: Vec<(String, bool)> = Vec::new();

    if fs::exists(&labels_folder).unwrap_or(false){
        let fichiers: Vec<String> = fs::read_dir(&labels_folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("csv"))
            .filter_map(|path| path.file_name()?.to_str().map(String::from))
            .collect();

        let set1: HashSet<&String> = fichiers.iter().collect();
        let set2: HashSet<&String> = selected_label_files_names_list.iter().collect();
        println!{"set1 : {:?}\n", set1}

        for line in set1 {
            if set2.contains(line) {
                label_files_list.push((line.to_string(), true));
            }
            else {
                label_files_list.push((line.to_string(), false))
            }
        }
            label_files_list.sort();

            return Ok(label_files_list);
    }

    Ok(vec![])
}

#[tauri::command(async)]
pub fn add_selected_label_files_list(
    selected_files_names_list: Vec<String>,
    state : State<'_, Arc<Mutex<SelectedLabelFiles>>>
)-> Result<(), tauri::Error> {
    let mut selected_label_files_list = state.lock().unwrap();

    selected_label_files_list.set(selected_files_names_list);

    println!("Ajouté à la liste : {:?}", selected_label_files_list);
    
    
    Ok(())
}

#[tauri::command(async)]
pub fn import_label_files(
    csv_paths: Vec<String>,
    app: tauri::AppHandle,
) -> Result<(), tauri::Error> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");

    if !fs::exists(&labels_folder).unwrap_or(false){
        fs::create_dir(&labels_folder)?;
    }
    
    for csv_path in &csv_paths {
        fs::copy(csv_path, &labels_folder)?;
        println!("copie de {:?} effectuée", csv_path);
    }

    Ok(())
}

pub fn labels_to_matrix(
    app: tauri::AppHandle,
    matrice: &mut FlowMatrix,
    label: State<'_, Arc<Mutex<SelectedLabelFiles>>>
) -> Result<(), std::io::Error> {
    let label_files_names_list = read_label_files_list(app.clone(), label).unwrap_or_default();
    let data_folder = app.path().app_data_dir().unwrap();
    let labels_folder = data_folder.join("labels");

    let selected_label_files_names: HashSet<String> = label_files_names_list.into_iter().filter(|(_, actif)| *actif).map(|(nom, _)| nom).collect();

    if fs::exists(&labels_folder).unwrap_or(false){
        let selected_label_files: Vec<PathBuf> = fs::read_dir(&labels_folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("csv"))
            .filter(|path| path.file_name().and_then(|e| e.to_str()).map(|name|selected_label_files_names.contains(name)).unwrap_or(false))
            .collect();
    

        for label_file in &selected_label_files {

            let file = match std::fs::read_to_string(&label_file) {
                Ok(csv_data) => csv_data,
                Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
                Err(error) => return Err(error.into()),
            };
            let labels: Vec<String> = file
                .lines()
                .map(|l| l.to_string())
                .collect();

            for label in labels {
                let Some((mac, ip, label)) = parse_label_row(&label) else {
                    continue;
                };

                matrice.add_label(mac.to_string(), ip, label);
            }
        }
    }

    Ok(())
}

#[tauri::command(async)]
pub fn remove_label_file(
    csv_file: String,
    app: AppHandle
)->Result<(), tauri::Error> {
    let data_folder = app.path().app_data_dir()?;
    let labels_folder = data_folder.join("labels");
    let label_file = labels_folder.join(csv_file);
    fs::remove_file(label_file)?;
    println!("File removed");
    Ok(())
}