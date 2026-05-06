use log::{error, info};
use packet_parser::PacketFlow;
use pcap::Capture;
use std::{sync::{Arc, Mutex}, 
    io::{ErrorKind, Write}, fs, 
    path::{Path, PathBuf},
    collections::HashSet
};
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::capture_handle::messages::capture::PacketMinimal, 
        flow_matrix::FlowMatrix,
        graph::GraphData,
    },
    setup::labels::parse_label_row
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
pub fn labels_file_list(
    app: tauri::AppHandle,
) -> Result<Vec<(String, bool)>, tauri::Error>{
    let dossier_data = app.path().app_data_dir()?;
    let dossier_labels = dossier_data.join("labels");
    let fichier_labels_file_list = dossier_labels.join("labels_file_list.txt");
    let mut couples: Vec<(String, bool)> = Vec::new();

    if fs::exists(&dossier_labels).unwrap_or(false){
        let fichiers: Vec<String> = fs::read_dir(&dossier_labels)?
            .filter_map(|entree| entree.ok())
            .map(|entree| entree.path())
            .filter(|chemin| chemin.is_file())
            .filter(|chemin| chemin.extension().and_then(|e| e.to_str()) == Some("csv"))
            .filter_map(|chemin| chemin.file_name()?.to_str().map(String::from))
            .collect();

        let contenu = fs::read_to_string(fichier_labels_file_list)?;
        let lignes: Vec<String> = contenu.lines().map(String::from).collect();

        let set1: HashSet<&String> = fichiers.iter().collect();
        let set2: HashSet<&String> = lignes.iter().collect();

        for ligne in set1 {
            if set2.contains(ligne) {
                couples.push((ligne.to_string(), true));
            }
            else {
                couples.push((ligne.to_string(), false))
            }

            println!("couples : {:?}\n", couples);
        }

        couples.sort();

        return Ok(couples);
    }

    Ok(vec![])
}

#[tauri::command(async)]
pub fn add_to_label_file_list(
    paths_list: Vec<String>,
    app: tauri::AppHandle,
)-> Result<(), tauri::Error> {
    let dossier_data = app.path().app_data_dir()?;
    let dossier_labels = dossier_data.join("labels");
    let fichier_labels_file_list = dossier_labels.join("labels_file_list.txt");

    if !fs::exists(&dossier_labels).unwrap_or(false){
        fs::create_dir(&dossier_labels)?;
    }

    let mut fichier_labels_file_list = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fichier_labels_file_list)?;

    for path in paths_list {
        fichier_labels_file_list.write_all(format!("{}\n", path).as_bytes())?;
    }

    println!("Ajouté à la liste : {:?}", fichier_labels_file_list);
    
    
    Ok(())
}

#[tauri::command(async)]
pub fn import_label_files(
    csv_paths: Vec<String>,
    app: tauri::AppHandle,
) -> Result<(), tauri::Error> {
    let dossier_data = app.path().app_data_dir()?;
    let dossier_labels = dossier_data.join("labels");

    if !fs::exists(&dossier_labels).unwrap_or(false){
        fs::create_dir(&dossier_labels)?;
    }

    for csv_path in csv_paths {
        let chemin = Path::new(&csv_path);
        fs::copy(&csv_path, &dossier_labels.join(chemin.file_name().unwrap()))?;
        println!("copie effectuée");
    }

    Ok(())
}

pub fn new_import_labels(
    app: tauri::AppHandle,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>
) -> Result<(), std::io::Error> {
    let mut state_label = state_label.lock().unwrap();
    let fichiers_choisis = labels_file_list(app.clone()).unwrap_or_default();
    let dossier_data = app.path().app_data_dir().unwrap();
    let dossier_labels = dossier_data.join("labels");

    let noms_actifs: HashSet<String> = fichiers_choisis.into_iter().filter(|(_, actif)| *actif).map(|(nom, _)| nom).collect();

    if fs::exists(&dossier_labels).unwrap_or(false){
        let fichiers: Vec<PathBuf> = fs::read_dir(&dossier_labels)?
            .filter_map(|entree| entree.ok())
            .map(|entree| entree.path())
            .filter(|chemin| chemin.is_file())
            .filter(|chemin| chemin.extension().and_then(|e| e.to_str()) == Some("csv"))
            .filter(|chemin| chemin.file_name().and_then(|e| e.to_str()).map(|nom|noms_actifs.contains(nom)).unwrap_or(false))
            .collect();
    

        for fichier in &fichiers {

            let csv = match std::fs::read_to_string(&fichier) {
            Ok(csv_data) => csv_data,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };

            let labels: Vec<String> = csv
                .lines()
                .map(|l| l.to_string())
                .collect();

            for label in labels {
                let Some((mac, ip, label)) = parse_label_row(&label) else {
                    continue;
                };

                state_label.add_label(mac.to_string(), ip, label);
            }
        }
    }

    Ok(())
}

#[tauri::command(async)]
pub fn remove_labels_file(
    csv_file: String,
    app: AppHandle
)->Result<(), tauri::Error> {
    println!("Entré dans remove_labels_file");
    let dossier_data = app.path().app_data_dir().unwrap();
    let dossier_labels = dossier_data.join("labels");
    let fichier_labels = dossier_labels.join(csv_file);
    fs::remove_file(fichier_labels)?;
    println!("Fichier supprimé");
    Ok(())
}

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
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
