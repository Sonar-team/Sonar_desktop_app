//! Conversion de fichiers PCAP en matrice de flux et graphe : lecture des
//! paquets, parsing, mise à jour de l'état et événements de progression.

use log::{error, info};
use packet_parser::LinkType;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;
#[cfg(feature = "capture_timing")]
use serde_json::json;
use sonar_flows_core::report::{PacketDisposition, PcapFileReport, classify_packet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(feature = "capture_timing")]
use std::time::Instant;
use tauri::{State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::{
            CaptureState,
            capture_handle::messages::capture::{CapturedPacket, CapturedPacketOwned},
        },
        flow_matrix::FlowMatrix,
        graph::{GraphData, GraphUpdate},
        labels_list::LabelStore,
    },
};

use super::labels::copy_labels_to_matrix;
use super::timing::ImportTimingLogger;
#[cfg(feature = "capture_timing")]
use super::timing::{ImportTimingSample, elapsed_ns_since, now_unix_ns, write_timing_or_disable};

/// Compte les paquets d'un fichier PCAP via le cœur partagé, qui distingue un
/// fichier tronqué/corrompu d'une fin normale ; la conversion d'erreur
/// (`From<SonarCoreError>`) préserve la distinction ouverture/lecture.
fn count_packets_in_pcap(file_path: &str) -> Result<usize, CaptureStateError> {
    Ok(sonar_flows_core::pcap::count_pcap_packets(Path::new(
        file_path,
    ))?)
}

/// Journal de timing des imports, actif seulement avec la feature
/// `capture_timing` (sinon `None`). Un échec d'ouverture désactive le journal
/// sans bloquer l'import.
fn new_timing_logger() -> Option<ImportTimingLogger> {
    #[cfg(feature = "capture_timing")]
    {
        match ImportTimingLogger::new() {
            Ok(logger) => Some(logger),
            Err(e) => {
                error!("Import timing log disabled: {}", e);
                None
            }
        }
    }
    #[cfg(not(feature = "capture_timing"))]
    {
        None
    }
}

fn send_started_event(on_event: &Channel<CaptureEvent<'_>>, link_type: &str) {
    if let Err(e) = on_event.send(CaptureEvent::Started {
        session_id: 0,
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 65536,
        link_type,
        protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    };
}

fn send_import_progress_event(
    on_event: &Channel<CaptureEvent<'_>>,
    file_name: &str,
    file_index: usize,
    files_total: usize,
    current: usize,
    total: usize,
) {
    if let Err(e) = on_event.send(CaptureEvent::ImportProgress {
        file_name,
        file_index,
        files_total,
        current,
        total,
    }) {
        error!("Erreur lors de l'envoi de ImportProgress: {:?}", e);
    }
}

/// Libellés des types de liaison (DLT) des fichiers à importer, dédupliqués
/// dans l'ordre de la liste. Un fichier illisible est ignoré ici : la
/// conversion elle-même échouera avec l'erreur précise.
fn import_link_types(pcap_paths: &[String]) -> String {
    let mut labels: Vec<String> = Vec::new();
    for path in pcap_paths {
        if let Ok(link_type) = sonar_flows_core::pcap::pcap_file_datalink(Path::new(path)) {
            let label = sonar_flows_core::pcap::datalink_label(link_type);
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    labels.join(", ")
}

/// Envoie l'événement `Stats` (compteurs de la barre de statut). Retourne
/// vrai si l'envoi a réussi. En import, les paquets « reçus » sont les
/// paquets lus du fichier ; les catégories de perte réseau n'existent pas.
fn send_stats_event(
    on_event: &Channel<CaptureEvent<'_>>,
    received: usize,
    processed: usize,
) -> bool {
    on_event
        .send(CaptureEvent::Stats {
            session_id: 0,
            received: received as u32,
            dropped: 0,
            if_dropped: 0,
            app_dropped: 0,
            rejected_truncated: 0,
            rejected_unsupported_link_type: 0,
            rejected_malformed: 0,
            processed: processed as u32,
        })
        .map_err(|e| {
            error!("Erreur lors de l'envoi de Stats: {:?}", e);
            e
        })
        .is_ok()
}

/// Labels source/destination d'un paquet possédé, résolus par la matrice
/// (clé MAC + IP, avec les replis de `FlowMatrix::get_label`).
fn lookup_flow_labels(
    matrice: &FlowMatrix,
    owned_packet: &CapturedPacketOwned,
) -> (Option<String>, Option<String>) {
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

    let link = sonar_flows_core::link::LinkView::of(&owned_packet.flow.data_link);
    (
        matrice.get_label(&link.source_mac, &source_ip),
        matrice.get_label(&link.destination_mac, &destination_ip),
    )
}

/// Applique un paquet possédé (un niveau de flux) à la matrice puis au graphe.
fn apply_owned_packet(
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    owned_packet: &CapturedPacketOwned,
) -> Vec<GraphUpdate> {
    matrice.update_flow(owned_packet);
    let (source_label, destination_label) = lookup_flow_labels(matrice, owned_packet);
    graph.add_packet_flow(
        &owned_packet.flow,
        source_label,
        destination_label,
        1,
        owned_packet.len as u64,
        owned_packet.encap_id.as_slice(),
    )
}

/// Traite un paquet lu du PCAP : parsing, mise à jour matrice + graphe pour
/// chaque niveau de flux (tunnels), stats périodiques.
#[cfg(not(feature = "capture_timing"))]
#[allow(clippy::too_many_arguments)]
fn process_packet(
    packet: &pcap::Packet<'_>,
    link_type: LinkType,
    packet_count: usize,
    total: usize,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
    report: &mut PcapFileReport,
) {
    let flow = match packet_parser::parse::parse(link_type, packet.data) {
        Ok(flow) => flow,
        Err(error) => {
            // Rapport qualité (#150) : un paquet illisible est classé par la
            // partition du cœur (troncature de capture, DLT inconnu, trame
            // malformée), jamais silencieusement ignoré — la même
            // classification que la CLI, pas une réimplémentation locale.
            report.record(classify_packet(
                packet.header.caplen,
                packet.header.len,
                Some(&error),
            ));
            return;
        }
    };
    report.record(PacketDisposition::Decoded);
    let packet_min = CapturedPacket::from_pcap(packet.header, flow);

    // Un paquet tunnelé (ex. CAPWAP) produit plusieurs niveaux de flux :
    // la ligne externe (tunnel) puis la (les) conversation(s) interne(s).
    for owned_packet in packet_min.to_owned_packets() {
        apply_owned_packet(matrice, graph, &owned_packet);
    }

    // Stats périodiques (optionnel)
    if packet_count.is_multiple_of(1000) || packet_count == total {
        send_stats_event(on_event, packet_count, matrice.row_count());
    }
}

/// Variante instrumentée de `process_packet` : mêmes traitements, avec mesure
/// de chaque étape et écriture d'une entrée JSONL par paquet échantillonné.
#[cfg(feature = "capture_timing")]
#[allow(clippy::too_many_arguments)]
fn process_packet_timed(
    packet: &pcap::Packet<'_>,
    link_type: LinkType,
    file_path: &str,
    packet_count: usize,
    total: usize,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
    timing_logger: &mut Option<ImportTimingLogger>,
    report: &mut PcapFileReport,
) {
    let timing_sample: Option<ImportTimingSample> = timing_logger
        .as_mut()
        .and_then(ImportTimingLogger::next_sample);
    let pipeline_start = timing_sample.map(|_| Instant::now());

    let parsed_flow = if timing_sample.is_some() {
        let mut parse_timing = ParseTiming::default();
        packet_parser::parse::parse_timed(link_type, packet.data, &mut parse_timing)
            .map(|flow| (flow, parse_timing))
            .map_err(|error| (error, parse_timing))
    } else {
        packet_parser::parse::parse(link_type, packet.data)
            .map(|flow| (flow, ParseTiming::default()))
            .map_err(|error| (error, ParseTiming::default()))
    };

    match parsed_flow {
        Ok((flow, parse_timing)) => {
            report.record(PacketDisposition::Decoded);
            let packet_min = CapturedPacket::from_pcap(packet.header, flow);

            let packet_owned_start = timing_sample.map(|_| Instant::now());
            let owned_packet = packet_min.to_owned_packet();
            let packet_owned_ns = packet_owned_start.map(elapsed_ns_since).unwrap_or(0);

            let matrix_update_start = timing_sample.map(|_| Instant::now());
            matrice.update_flow(&owned_packet);
            let matrix_count = matrice.row_count();
            let matrix_update_ns = matrix_update_start.map(elapsed_ns_since).unwrap_or(0);

            let label_lookup_start = timing_sample.map(|_| Instant::now());
            let (source_label, destination_label) = lookup_flow_labels(matrice, &owned_packet);
            let label_lookup_ns = label_lookup_start.map(elapsed_ns_since).unwrap_or(0);

            let graph_update_start = timing_sample.map(|_| Instant::now());
            let graph_updates = graph.add_packet_flow(
                &owned_packet.flow,
                source_label,
                destination_label,
                1,
                owned_packet.len as u64,
                owned_packet.encap_id.as_slice(),
            );
            let graph_update_ns = graph_update_start.map(elapsed_ns_since).unwrap_or(0);

            // Niveaux internes des tunnels (non instrumentés).
            for inner in packet_min.to_owned_packets().into_iter().skip(1) {
                apply_owned_packet(matrice, graph, &inner);
            }

            let mut stats_ipc_ns = 0;
            let mut stats_ipc_sent = false;
            let mut stats_ipc_ok = false;
            if packet_count.is_multiple_of(1000) || packet_count == total {
                stats_ipc_sent = true;
                let stats_ipc_start = timing_sample.map(|_| Instant::now());
                stats_ipc_ok = send_stats_event(on_event, packet_count, matrix_count);
                stats_ipc_ns = stats_ipc_start.map(elapsed_ns_since).unwrap_or(0);
            }

            if let (Some(sample), Some(start)) = (timing_sample, pipeline_start) {
                write_timing_or_disable(
                    timing_logger,
                    json!({
                        "event": "import_packet_timing",
                        "ts_unix_ns": now_unix_ns(),
                        "file_path": file_path,
                        "seq": sample.seq,
                        "packet_index": packet_count,
                        "total_packets": total,
                        "sample_rate": sample.sample_rate,
                        "caplen": packet.header.caplen,
                        "len": packet.header.len,
                        "parse_l2_ns": parse_timing.l2_ns,
                        "parse_l3_ns": parse_timing.l3_ns,
                        "parse_l4_ns": parse_timing.l4_ns,
                        "parse_l7_ns": parse_timing.l7_ns,
                        "parse_total_ns": parse_timing.total_ns,
                        "packet_owned_ns": packet_owned_ns,
                        "matrix_update_ns": matrix_update_ns,
                        "label_lookup_ns": label_lookup_ns,
                        "graph_update_ns": graph_update_ns,
                        "graph_updates": graph_updates.len(),
                        "stats_ipc_ns": stats_ipc_ns,
                        "stats_ipc_sent": stats_ipc_sent,
                        "stats_ipc_ok": stats_ipc_ok,
                        "matrix_count": matrix_count,
                        "pipeline_total_ns": elapsed_ns_since(start)
                    }),
                    "packet",
                );
            }
        }
        Err((parse_error, parse_timing)) => {
            report.record(classify_packet(
                packet.header.caplen,
                packet.header.len,
                Some(&parse_error),
            ));

            if let (Some(sample), Some(start)) = (timing_sample, pipeline_start) {
                write_timing_or_disable(
                    timing_logger,
                    json!({
                        "event": "import_parse_error_timing",
                        "ts_unix_ns": now_unix_ns(),
                        "file_path": file_path,
                        "seq": sample.seq,
                        "packet_index": packet_count,
                        "total_packets": total,
                        "sample_rate": sample.sample_rate,
                        "caplen": packet.header.caplen,
                        "len": packet.header.len,
                        "error": parse_error.to_string(),
                        "parse_l2_ns": parse_timing.l2_ns,
                        "parse_l3_ns": parse_timing.l3_ns,
                        "parse_l4_ns": parse_timing.l4_ns,
                        "parse_l7_ns": parse_timing.l7_ns,
                        "parse_total_ns": parse_timing.total_ns,
                        "pipeline_total_ns": elapsed_ns_since(start)
                    }),
                    "parse error",
                );
            }
        }
    }
}

/// Convertit un fichier PCAP : compte les paquets (pour la progression), les
/// rejoue un à un dans la matrice et le graphe, puis signale la fin du fichier.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_pcap_file(
    file_path: &str,
    file_index: usize,
    files_total: usize,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
    timing_logger: &mut Option<ImportTimingLogger>,
    cancel: &AtomicBool,
) -> Result<(), CaptureStateError> {
    #[cfg(not(feature = "capture_timing"))]
    let _ = timing_logger;
    #[cfg(feature = "capture_timing")]
    let file_start = Instant::now();
    // Annulation entre deux fichiers : le fichier suivant n'est pas ouvert.
    if cancel.load(Ordering::Relaxed) {
        return Err(PcapImportError::Cancelled(file_path.to_string()).into());
    }
    // Un DLT sans décodeur échoue ici, avant toute lecture de paquet : le
    // fichier ne doit pas être parsé « comme si » c'était de l'Ethernet (#150).
    let link_type = sonar_flows_core::pcap::supported_parser_link_type(Path::new(file_path))?;
    // Un relevé = un réseau = un DLT : un fichier d'un autre type de liaison
    // que les précédents est refusé (fusion inter-DLT, arbitrage 14/07/2026).
    matrice.bind_link_type(link_type, Path::new(file_path))?;
    #[cfg(feature = "capture_timing")]
    let count_start = Instant::now();
    let total = count_packets_in_pcap(file_path)?;
    #[cfg(feature = "capture_timing")]
    let count_packets_ns = elapsed_ns_since(count_start);

    info!(
        "[handle_pcap_file] {} : {} paquets détectés",
        file_path, total
    );
    send_import_progress_event(on_event, file_path, file_index, files_total, 0, total);

    let mut packet_count: usize = 0;
    #[cfg(feature = "capture_timing")]
    let process_start = Instant::now();
    let mut report = PcapFileReport::default();

    // La boucle de lecture vient du cœur partagé : fin normale distinguée
    // d'une erreur en cours de fichier (tronqué, corrompu), propagée pour que
    // l'import échoue explicitement au lieu de produire une matrice partielle
    // en silence.
    sonar_flows_core::pcap::for_each_raw_packet(Path::new(file_path), |packet| {
        // Annulation en cours de fichier : la boucle du cœur ne peut pas être
        // interrompue (closure sans retour), les paquets restants sont donc
        // drainés sans parsing ni mise à jour — la partie coûteuse — et
        // l'erreur typée est retournée après la boucle.
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        packet_count += 1;
        if packet_count.is_multiple_of(1000) || packet_count == total {
            send_import_progress_event(
                on_event,
                file_path,
                file_index,
                files_total,
                packet_count,
                total,
            );
        }

        #[cfg(feature = "capture_timing")]
        process_packet_timed(
            packet,
            link_type,
            file_path,
            packet_count,
            total,
            matrice,
            graph,
            on_event,
            timing_logger,
            &mut report,
        );
        #[cfg(not(feature = "capture_timing"))]
        process_packet(
            packet,
            link_type,
            packet_count,
            total,
            matrice,
            graph,
            on_event,
            &mut report,
        );
    })?;

    // Fichier drainé après annulation : ni Finished ni comptabilité, la
    // commande échoue avec l'erreur typée et l'état partagé reste intact.
    if cancel.load(Ordering::Relaxed) {
        info!(
            "[handle_pcap_file] {} : import annulé par l'opérateur après {} paquets",
            file_path, packet_count
        );
        return Err(PcapImportError::Cancelled(file_path.to_string()).into());
    }

    #[cfg(feature = "capture_timing")]
    let process_ns = elapsed_ns_since(process_start);
    #[cfg(feature = "capture_timing")]
    let finished_ipc_start = Instant::now();
    // Le total émis est la somme des catégories (`read()`) : l'équation du
    // bilan tient par construction jusque dans l'IPC (#150).
    let finished_send_result = on_event.send(CaptureEvent::Finished {
        file_name: file_path,
        packet_total_count: report.read(),
        integrated_count: report.decoded,
        rejected_truncated_count: report.rejected_truncated,
        rejected_unsupported_link_type_count: report.rejected_unsupported_link_type,
        rejected_malformed_count: report.rejected_malformed,
        matrix_total_count: matrice.row_count(),
    });
    if let Err(e) = &finished_send_result {
        error!("Erreur lors de l'envoi de Finished: {:?}", e);
    };
    #[cfg(feature = "capture_timing")]
    {
        let finished_ipc_ns = elapsed_ns_since(finished_ipc_start);
        write_timing_or_disable(
            timing_logger,
            json!({
                "event": "import_file_timing",
                "ts_unix_ns": now_unix_ns(),
                "file_path": file_path,
                "total_packets": total,
                "read_packets": packet_count,
                "decoded": report.decoded,
                "rejected_truncated": report.rejected_truncated,
                "rejected_unsupported_link_type": report.rejected_unsupported_link_type,
                "rejected_malformed": report.rejected_malformed,
                "matrix_count": matrice.row_count(),
                "graph_nodes": graph.nodes.len(),
                "graph_edges": graph.edges.len(),
                "count_packets_ns": count_packets_ns,
                "process_ns": process_ns,
                "finished_ipc_ns": finished_ipc_ns,
                "finished_ipc_ok": finished_send_result.is_ok(),
                "file_total_ns": elapsed_ns_since(file_start)
            }),
            "file",
        );
    }

    info!(
        "[handle_pcap_file] {} : {} paquets lus = {} intégrés + {} tronqués \
         + {} DLT non supportés + {} malformés, {} lignes matrice",
        file_path,
        report.read(),
        report.decoded,
        report.rejected_truncated,
        report.rejected_unsupported_link_type,
        report.rejected_malformed,
        matrice.row_count()
    );
    Ok(())
}

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    pcap_paths: Vec<String>,
    on_event: Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    // Refus avant la réservation, le premier événement, la lecture des
    // labels et toute mutation : une liste vide ne représente jamais une
    // demande de remplacement du relevé courant (#167).
    if pcap_paths.is_empty() {
        return Err(PcapImportError::MissingInput("l'import PCAP".to_string()).into());
    }

    // Un même fichier sélectionné deux fois (doublon, chemin relatif, lien
    // symbolique) doublerait ses paquets dans la matrice : la déduplication
    // par identité réelle est faite ici, seul endroit qui puisse interroger
    // le système de fichiers (#161).
    let pcap_paths = super::dedupe_paths_by_identity(&pcap_paths);

    // Import et capture sont mutuellement exclusifs : la conversion
    // remplacerait la matrice et le graphe pendant que le pipeline les
    // alimente. La phase `Importing` est réservée atomiquement et détenue
    // jusqu'à la fin de la commande, swap inclus (#139) — un démarrage de
    // capture pendant la conversion est refusé par la machine d'état.
    let import_guard = crate::state::capture::ImportGuard::acquire(
        capture_state.inner(),
        "import de fichiers PCAP",
    )?;

    let mut timing_logger = new_timing_logger();
    #[cfg(feature = "capture_timing")]
    let command_start = Instant::now();

    info!(
        "[convert_from_pcap_list] COMMAND CALLED avec pcap_paths = {:?}",
        pcap_paths
    );

    send_started_event(&on_event, &import_link_types(&pcap_paths));

    // Copie des labels sous verrou court : le store n'est pas tenu pendant
    // la conversion.
    let labels = LabelStore {
        rows: label_store.lock()?.get().clone(),
    };

    // Import transactionnel : matrice et graphe sont construits en local et
    // l'état partagé n'est remplacé qu'en cas de succès complet. Une erreur
    // (fichier illisible, tronqué…) ou une annulation de l'opérateur
    // préserve l'état courant, et les verrous ne sont tenus que le temps du
    // swap au lieu de toute la conversion.
    let cancel = import_guard.cancel_token();
    let (new_matrix, new_graph) = build_matrix_and_graph_from_pcaps(
        &pcap_paths,
        &labels,
        &on_event,
        &mut timing_logger,
        &cancel,
    )?;

    info!("[convert_from_pcap_list] FIN traitement liste PCAP");

    import_guard.verify_current("commit de l'import PCAP")?;
    // Le commit est certain à partir d'ici : relevé marqué modifié avant de
    // prendre les verrous de données (#166, #159).
    capture_state.lock()?.mark_dirty();
    let mut matrice_guard = matrice.lock()?;
    let mut graph_guard = graph.lock()?;
    #[cfg(feature = "capture_timing")]
    let swap_start = Instant::now();
    *matrice_guard = new_matrix;
    *graph_guard = new_graph;
    #[cfg(feature = "capture_timing")]
    let reset_ns = elapsed_ns_since(swap_start);

    // 🔥 snapshot complet envoyé sur le channel
    #[cfg(feature = "capture_timing")]
    let snapshot_build_start = Instant::now();
    let snapshot: GraphData = graph_guard.get_all_graph_data(); // doit renvoyer un GraphData possédé
    #[cfg(feature = "capture_timing")]
    let snapshot_build_ns = elapsed_ns_since(snapshot_build_start);

    #[cfg(feature = "capture_timing")]
    let snapshot_ipc_start = Instant::now();
    let snapshot_send_result = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    });
    if let Err(e) = &snapshot_send_result {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }
    #[cfg(feature = "capture_timing")]
    {
        let snapshot_ipc_ns = elapsed_ns_since(snapshot_ipc_start);
        if let Some(logger) = timing_logger.as_mut()
            && let Err(e) = logger.write_value(json!({
                "event": "import_snapshot_timing",
                "ts_unix_ns": now_unix_ns(),
                "files": pcap_paths.len(),
                "matrix_count": matrice_guard.row_count(),
                "graph_nodes": snapshot.nodes.len(),
                "graph_edges": snapshot.edges.len(),
                "snapshot_build_ns": snapshot_build_ns,
                "snapshot_ipc_ns": snapshot_ipc_ns,
                "snapshot_ipc_ok": snapshot_send_result.is_ok(),
                "reset_ns": reset_ns,
                "command_total_ns": elapsed_ns_since(command_start)
            }))
        {
            error!(
                "Import timing log disabled after snapshot write error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Construit matrice + graphe depuis les fichiers PCAP, sans toucher à
/// l'état partagé : l'appelant ne remplace l'état qu'en cas de succès complet
/// (import transactionnel).
fn build_matrix_and_graph_from_pcaps(
    pcap_paths: &[String],
    labels: &LabelStore,
    on_event: &Channel<CaptureEvent<'_>>,
    timing_logger: &mut Option<ImportTimingLogger>,
    cancel: &AtomicBool,
) -> Result<(FlowMatrix, GraphData), CaptureStateError> {
    let mut matrix = FlowMatrix::new();
    let mut graph = GraphData::new();
    copy_labels_to_matrix(labels, &mut matrix)?;

    for (file_index, pcap_path) in pcap_paths.iter().enumerate() {
        info!("[convert_from_pcap_list] Traitement de {}", pcap_path);
        handle_pcap_file(
            pcap_path,
            file_index + 1,
            pcap_paths.len(),
            &mut matrix,
            &mut graph,
            on_event,
            timing_logger,
            cancel,
        )?;
    }

    Ok((matrix, graph))
}

/// Demande l'annulation coopérative de l'import PCAP en cours : le fichier
/// courant est drainé sans analyse et les fichiers suivants ne sont pas
/// ouverts. Retourne vrai si un import actif va être interrompu, faux sinon
/// (course avec la fin d'import : sans effet, l'opérateur n'a rien à faire).
#[tauri::command]
pub fn cancel_import(
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<bool, CaptureStateError> {
    let cancelled = capture_state.lock()?.request_import_cancel();
    info!("[cancel_import] demande d'annulation, import actif : {cancelled}");
    Ok(cancelled)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{TempDir, tshark_corpus, tunnel_pcap};
    use super::*;
    use crate::errors::import::PcapImportError;
    use crate::state::capture::capture_status::CapturePhase;
    use crate::state::flow_matrix::FlowMatrixRow;
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn pcap_invoke_request(paths: Vec<String>, channel_id: u32) -> InvokeRequest {
        InvokeRequest {
            cmd: "convert_from_pcap_list".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(any(windows, target_os = "android")) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(serde_json::json!({
                "pcapPaths": paths,
                "onEvent": format!("__CHANNEL__:{channel_id}"),
            })),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    /// PCAP dont l'en-tête de paquet annonce 100 octets mais n'en fournit
    /// que 10 : fichier tronqué typique (copie interrompue, disque plein).
    fn write_truncated_pcap(path: &std::path::Path) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes()); // magic
        bytes.extend_from_slice(&2u16.to_le_bytes()); // version majeure
        bytes.extend_from_slice(&4u16.to_le_bytes()); // version mineure
        bytes.extend_from_slice(&0i32.to_le_bytes()); // thiszone
        bytes.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
        bytes.extend_from_slice(&65_535u32.to_le_bytes()); // snaplen
        bytes.extend_from_slice(&1u32.to_le_bytes()); // linktype Ethernet
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_sec
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
        bytes.extend_from_slice(&100u32.to_le_bytes()); // caplen annoncé
        bytes.extend_from_slice(&100u32.to_le_bytes()); // len annoncé
        bytes.extend_from_slice(&[0u8; 10]); // ... mais 10 octets seulement
        fs::write(path, &bytes).unwrap();
    }

    /// Un fichier tronqué doit produire une erreur explicite, pas une fin de
    /// fichier silencieuse (matrice partielle).
    #[test]
    fn truncated_pcap_returns_read_error() {
        let dir = TempDir::new("sonar_test_truncated_pcap");
        let path = dir.path().join("tronque.pcap");
        write_truncated_pcap(&path);

        match count_packets_in_pcap(path.to_str().unwrap()) {
            Err(CaptureStateError::Import(PcapImportError::ReadPacketError(file, _))) => {
                assert!(file.ends_with("tronque.pcap"));
            }
            other => panic!("attendu ReadPacketError, obtenu {other:?}"),
        }
    }

    /// L'import est transactionnel : le builder échoue sans avoir touché à
    /// l'état partagé (il n'y a pas accès), l'appelant garde donc la matrice
    /// courante en cas d'erreur.
    #[test]
    fn build_from_pcaps_fails_cleanly_on_bad_file() {
        let dir = TempDir::new("sonar_test_transactional_import");
        let bad = dir.path().join("tronque.pcap");
        write_truncated_pcap(&bad);
        let labels = crate::state::labels_list::LabelStore::new();
        let on_event = Channel::new(|_| Ok(()));

        let result = build_matrix_and_graph_from_pcaps(
            &[bad.to_str().unwrap().to_string()],
            &labels,
            &on_event,
            &mut None,
            &AtomicBool::new(false),
        );

        assert!(result.is_err(), "fichier tronqué -> erreur propagée");

        let missing = build_matrix_and_graph_from_pcaps(
            &["/inexistant/capture.pcap".to_string()],
            &labels,
            &on_event,
            &mut None,
            &AtomicBool::new(false),
        );
        assert!(missing.is_err(), "fichier absent -> erreur propagée");
    }

    /// Test ciblé du pipeline appelé par `convert_from_pcap_list` avec les
    /// quatre DLT réellement supportés. Chaque fichier est importé dans une
    /// transaction séparée puisque la fusion inter-DLT est refusée par contrat.
    #[test]
    fn convert_from_pcap_list_pipeline_imports_real_multi_dlt_corpus() {
        let labels = crate::state::labels_list::LabelStore::new();
        let on_event = Channel::new(|_| Ok(()));
        for (name, expected_link_type, expected_packets) in [
            ("vlan.pcap", LinkType::ETHERNET, 7_u64),
            ("raw_ip.pcap", LinkType::RAW, 15),
            ("linux_sll.pcap", LinkType::LINUX_SLL, 2_702),
            ("linux_sll2.pcap", LinkType::LINUX_SLL2, 779),
        ] {
            let path = tshark_corpus(name);
            let (matrix, graph) = build_matrix_and_graph_from_pcaps(
                &[path.to_string_lossy().into_owned()],
                &labels,
                &on_event,
                &mut None,
                &AtomicBool::new(false),
            )
            .unwrap_or_else(|error| panic!("import Tauri de {name}: {error}"));

            assert_eq!(matrix.link_type, Some(expected_link_type), "{name}");
            assert_eq!(
                matrix
                    .to_flat_vec()
                    .iter()
                    .map(|row| row.count)
                    .sum::<u64>(),
                expected_packets,
                "comptabilité du wrapper Tauri pour {name}"
            );
            assert!(matrix.row_count() > 0, "matrice vide pour {name}");
            assert!(!graph.nodes.is_empty(), "graphe vide pour {name}");
        }
    }

    #[test]
    fn pcap_import_emits_initial_and_final_progress() {
        let pcap_path = tunnel_pcap();
        let progress_events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = progress_events.clone();
        let on_event = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                let value: serde_json::Value = serde_json::from_str(&json).unwrap();
                if value["event"] == "importProgress" {
                    captured.lock().unwrap().push(value);
                }
            }
            Ok(())
        });
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();

        handle_pcap_file(
            pcap_path.to_str().unwrap(),
            2,
            3,
            &mut matrix,
            &mut graph,
            &on_event,
            &mut None,
            &AtomicBool::new(false),
        )
        .unwrap();

        let events = progress_events.lock().unwrap();
        assert!(events.len() >= 2, "progression initiale et finale attendue");
        let first = events.first().unwrap();
        let last = events.last().unwrap();
        assert_eq!(first["data"]["fileIndex"].as_u64(), Some(2));
        assert_eq!(first["data"]["filesTotal"].as_u64(), Some(3));
        assert_eq!(first["data"]["current"].as_u64(), Some(0));
        assert_eq!(last["data"]["current"], last["data"]["total"]);
        assert!(
            last["data"]["total"]
                .as_u64()
                .is_some_and(|total| total > 0)
        );
    }

    /// Test de l'adaptateur Tauri complet demandé par #168 : la commande est
    /// appelée par son handler IPC (pas par un helper Rust), sur une vraie
    /// capture SLL assez longue pour émettre plusieurs jalons. Le channel est
    /// intercepté comme le ferait le frontend afin de prouver simultanément
    /// la monotonie de la progression et l'exclusivité de `ImportGuard`.
    #[test]
    fn tauri_pcap_command_preserves_progress_and_import_exclusivity() {
        let matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let labels = Arc::new(Mutex::new(LabelStore::new()));
        let capture_state = Arc::new(Mutex::new(CaptureState::new()));
        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let exclusivity_checked = Arc::new(AtomicBool::new(false));

        let intercepted_events = Arc::clone(&events);
        let intercepted_state = Arc::clone(&capture_state);
        let intercepted_exclusivity = Arc::clone(&exclusivity_checked);
        let app = mock_builder()
            .channel_interceptor(move |_webview, _callback, _index, body| {
                if let InvokeResponseBody::Json(json) = body {
                    let value: serde_json::Value =
                        serde_json::from_str(json).expect("événement Tauri JSON");
                    if value["event"] == "started" {
                        let concurrent = crate::state::capture::ImportGuard::acquire(
                            &intercepted_state,
                            "import PCAP concurrent de test",
                        );
                        assert!(
                            matches!(
                                concurrent,
                                Err(CaptureStateError::InvalidTransition { ref from, .. })
                                    if from == "importing"
                            ),
                            "le guard de la commande doit précéder le premier événement"
                        );
                        intercepted_exclusivity.store(true, Ordering::SeqCst);
                    }
                    intercepted_events.lock().unwrap().push(value);
                }
                true
            })
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&labels))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![convert_from_pcap_list])
            .build(mock_context(noop_assets()))
            .expect("application Tauri simulée");
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("webview Tauri simulée");

        let capture = tshark_corpus("linux_sll.pcap");
        let response = get_ipc_response(
            &webview,
            pcap_invoke_request(vec![capture.to_string_lossy().into_owned()], 168),
        );
        assert!(response.is_ok(), "réponse IPC d'import : {response:?}");

        let progress = events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event["event"] == "importProgress")
            .map(|event| {
                (
                    event["data"]["current"].as_u64().unwrap(),
                    event["data"]["total"].as_u64().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(progress.first().copied(), Some((0, 2_702)), "jalon initial");
        assert_eq!(
            progress.last().copied(),
            Some((2_702, 2_702)),
            "jalon final"
        );
        assert!(progress.len() >= 4, "jalons 0, 1000, 2000 et fin attendus");
        assert!(
            progress.windows(2).all(|pair| pair[0].0 <= pair[1].0),
            "la progression IPC doit être monotone : {progress:?}"
        );
        assert!(
            exclusivity_checked.load(Ordering::SeqCst),
            "l'exclusion mutuelle doit être observée pendant la commande"
        );
        assert_eq!(capture_state.lock().unwrap().phase, CapturePhase::Idle);
        assert_eq!(matrix.lock().unwrap().link_type, Some(LinkType::LINUX_SLL));
        assert!(!graph.lock().unwrap().nodes.is_empty());
    }

    /// La frontière IPC ne doit pas aplatir l'erreur typée du cœur en chaîne
    /// générique : le frontend reçoit toujours `import/unsupportedLinkType`,
    /// avec le chemin fautif, et aucun état partiel n'est publié.
    #[test]
    fn tauri_pcap_command_preserves_typed_core_error_for_frontend() {
        let matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let labels = Arc::new(Mutex::new(LabelStore::new()));
        let capture_state = Arc::new(Mutex::new(CaptureState::new()));
        let app = mock_builder()
            .channel_interceptor(|_webview, _callback, _index, _body| true)
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&labels))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![convert_from_pcap_list])
            .build(mock_context(noop_assets()))
            .expect("application Tauri simulée");
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("webview Tauri simulée");

        let capture = tshark_corpus("unsupported_ieee80211.pcapng");
        let error = get_ipc_response(
            &webview,
            pcap_invoke_request(vec![capture.to_string_lossy().into_owned()], 169),
        )
        .expect_err("le DLT IEEE 802.11 doit être refusé");

        assert_eq!(error["kind"], "import");
        assert_eq!(error["message"]["kind"], "unsupportedLinkType");
        assert!(
            error["message"]["message"][0]
                .as_str()
                .is_some_and(|path| path.ends_with("unsupported_ieee80211.pcapng")),
            "le chemin du cœur doit traverser l'IPC : {error}"
        );
        assert_eq!(capture_state.lock().unwrap().phase, CapturePhase::Idle);
        assert_eq!(matrix.lock().unwrap().row_count(), 0);
        assert!(graph.lock().unwrap().nodes.is_empty());
    }

    /// Une liste vide est une erreur de contrat, pas un relevé vide à
    /// committer. Le refus IPC doit précéder Started et préserver toutes les
    /// composantes de la session courante (#167).
    #[test]
    fn tauri_pcap_command_refuses_empty_list_without_touching_session() {
        let mut seeded_matrix = FlowMatrix::new();
        let mut seeded_graph = GraphData::new();
        let seed_channel = Channel::new(|_| Ok(()));
        let capture = tunnel_pcap();
        handle_pcap_file(
            capture.to_str().unwrap(),
            1,
            1,
            &mut seeded_matrix,
            &mut seeded_graph,
            &seed_channel,
            &mut None,
            &AtomicBool::new(false),
        )
        .unwrap();
        seeded_matrix.add_label(
            "aa:bb:cc:dd:ee:ff".to_string(),
            "192.0.2.1".to_string(),
            "sentinelle matrice".to_string(),
        );

        let mut seeded_labels = LabelStore::new();
        seeded_labels.add((
            "aa:bb:cc:dd:ee:ff".to_string(),
            "192.0.2.1".to_string(),
            "sentinelle store".to_string(),
        ));
        let mut seeded_capture_state = CaptureState::new();
        seeded_capture_state.session_id = 37;
        seeded_capture_state.filter = Some("tcp port 443".to_string());
        seeded_capture_state.config.device_name = "interface-sentinelle".to_string();

        let matrix = Arc::new(Mutex::new(seeded_matrix));
        let graph = Arc::new(Mutex::new(seeded_graph));
        let labels = Arc::new(Mutex::new(seeded_labels));
        let capture_state = Arc::new(Mutex::new(seeded_capture_state));
        let before_matrix = matrix.lock().unwrap().to_flat_vec();
        let before_matrix_labels = matrix.lock().unwrap().label.clone();
        let before_graph = serde_json::to_value(&*graph.lock().unwrap()).unwrap();
        let before_labels = labels.lock().unwrap().rows.clone();
        let before_config = serde_json::to_value(&capture_state.lock().unwrap().config).unwrap();
        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let intercepted_events = Arc::clone(&events);

        let app = mock_builder()
            .channel_interceptor(move |_webview, _callback, _index, body| {
                if let InvokeResponseBody::Json(json) = body {
                    intercepted_events
                        .lock()
                        .unwrap()
                        .push(serde_json::from_str(json).unwrap());
                }
                true
            })
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&labels))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![convert_from_pcap_list])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let error = get_ipc_response(&webview, pcap_invoke_request(vec![], 167))
            .expect_err("la liste PCAP vide doit être refusée");

        assert_eq!(error["kind"], "import");
        assert_eq!(error["message"]["kind"], "missingInput");
        assert!(events.lock().unwrap().is_empty(), "aucun événement émis");
        assert_eq!(matrix.lock().unwrap().to_flat_vec(), before_matrix);
        assert_eq!(matrix.lock().unwrap().label, before_matrix_labels);
        assert_eq!(
            serde_json::to_value(&*graph.lock().unwrap()).unwrap(),
            before_graph
        );
        assert_eq!(labels.lock().unwrap().rows, before_labels);
        let locked_state = capture_state.lock().unwrap();
        assert_eq!(locked_state.phase, CapturePhase::Idle);
        assert_eq!(locked_state.session_id, 37);
        assert_eq!(locked_state.filter.as_deref(), Some("tcp port 443"));
        assert_eq!(
            serde_json::to_value(&locked_state.config).unwrap(),
            before_config
        );
    }

    /// Une annulation demandée avant un fichier refuse de l'ouvrir : erreur
    /// typée `Cancelled`, aucun événement de progression émis.
    #[test]
    fn cancelled_before_file_skips_it_entirely() {
        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let captured = Arc::clone(&events);
        let on_event = Channel::new(move |body| {
            if let InvokeResponseBody::Json(json) = body {
                captured
                    .lock()
                    .unwrap()
                    .push(serde_json::from_str(&json).unwrap());
            }
            Ok(())
        });
        let cancel = AtomicBool::new(true);

        let result = build_matrix_and_graph_from_pcaps(
            &[tunnel_pcap().to_string_lossy().into_owned()],
            &crate::state::labels_list::LabelStore::new(),
            &on_event,
            &mut None,
            &cancel,
        );

        match result {
            Err(CaptureStateError::Import(PcapImportError::Cancelled(file))) => {
                assert!(file.ends_with("ndpi_capwap.pcap"));
            }
            Err(other) => panic!("attendu Cancelled, obtenu {other:?}"),
            Ok(_) => panic!("attendu Cancelled, obtenu un import réussi"),
        }
        assert!(events.lock().unwrap().is_empty(), "aucun événement émis");
    }

    /// Une annulation en cours de fichier draine les paquets restants sans
    /// les intégrer, n'émet pas `Finished` et retourne l'erreur typée. La
    /// matrice locale abandonnée ne contient que les paquets déjà traités —
    /// l'état partagé, lui, n'est jamais touché (import transactionnel).
    #[test]
    fn cancelled_mid_file_drains_and_returns_typed_error() {
        let cancel = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let cancel_on_first_progress = Arc::clone(&cancel);
        let captured = Arc::clone(&events);
        let on_event = Channel::new(move |body| {
            if let InvokeResponseBody::Json(json) = body {
                let value: serde_json::Value = serde_json::from_str(&json).unwrap();
                // Annule dès le premier jalon de progression, comme le ferait
                // l'opérateur via `cancel_import`.
                if value["event"] == "importProgress" {
                    cancel_on_first_progress.store(true, Ordering::Relaxed);
                }
                captured.lock().unwrap().push(value);
            }
            Ok(())
        });
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();

        let result = handle_pcap_file(
            tunnel_pcap().to_str().unwrap(),
            1,
            1,
            &mut matrix,
            &mut graph,
            &on_event,
            &mut None,
            &cancel,
        );

        match result {
            Err(CaptureStateError::Import(PcapImportError::Cancelled(file))) => {
                assert!(file.ends_with("ndpi_capwap.pcap"));
            }
            other => panic!("attendu Cancelled, obtenu {other:?}"),
        }
        let events = events.lock().unwrap();
        assert!(
            events.iter().all(|e| e["event"] != "finished"),
            "un fichier annulé ne doit pas être déclaré terminé : {events:?}"
        );
        // Le jalon initial (current = 0) est émis avant l'annulation, puis
        // plus rien : le drainage n'annonce pas de progression.
        assert!(
            events
                .iter()
                .filter(|e| e["event"] == "importProgress")
                .all(|e| e["data"]["current"].as_u64() == Some(0)),
            "aucune progression après l'annulation : {events:?}"
        );
    }

    /// `cancel_import` n'a d'effet que pendant un import : hors import la
    /// demande est ignorée (course avec la fin d'import), et chaque nouvelle
    /// réservation repart avec un jeton vierge.
    #[test]
    fn cancel_request_only_affects_active_import_and_resets_per_reservation() {
        let state = Arc::new(Mutex::new(CaptureState::new()));

        // Hors import : demande ignorée.
        assert!(!state.lock().unwrap().request_import_cancel());

        // Pendant un import : demande prise en compte, jeton levé.
        let guard = crate::state::capture::ImportGuard::acquire(&state, "test annulation").unwrap();
        assert!(!guard.cancel_token().load(Ordering::Relaxed));
        assert!(state.lock().unwrap().request_import_cancel());
        assert!(guard.cancel_token().load(Ordering::Relaxed));
        drop(guard);

        // Réservation suivante : le jeton de l'annulation périmée est remis
        // à zéro par l'acquisition.
        let next = crate::state::capture::ImportGuard::acquire(&state, "test annulation").unwrap();
        assert!(!next.cancel_token().load(Ordering::Relaxed));
    }

    /// Rejoue l'import PCAP réel et vérifie la comptabilité des tunnels :
    /// pour chaque tunnel, la somme des paquets attribués aux lignes internes
    /// (via la colonne `encap_id`, forme `id` ou `id:n|…`) doit être égale au
    /// compteur des lignes externes CAPWAP (aller + retour), sans orphelin.
    #[test]
    fn import_pcap_tunnel_counts_are_balanced() {
        let pcap_path = tunnel_pcap();
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();
        let on_event = Channel::new(|_| Ok(()));

        handle_pcap_file(
            pcap_path.to_str().unwrap(),
            1,
            1,
            &mut matrix,
            &mut graph,
            &on_event,
            &mut None,
            &AtomicBool::new(false),
        )
        .unwrap();

        // Ventile la colonne `encap_id` d'une ligne : `id` nu = tout le
        // compteur de la ligne, `id:n` = n paquets pour ce tunnel.
        fn tunnel_counts(row: &FlowMatrixRow) -> Vec<(String, u64)> {
            row.encap_id
                .split('|')
                .filter(|e| !e.is_empty())
                .map(|element| match element.split_once(':') {
                    Some((id, n)) => (id.to_string(), n.parse::<u64>().unwrap()),
                    None => (element.to_string(), row.count),
                })
                .collect()
        }

        let rows = matrix.to_flat_vec();
        assert_eq!(rows.len(), matrix.row_count(), "une ligne par flux");

        let mut outer: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut inner: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for row in &rows {
            let target = if row.application_protocol.as_deref() == Some("CAPWAP") {
                &mut outer
            } else {
                &mut inner
            };
            for (id, count) in tunnel_counts(row) {
                *target.entry(id).or_default() += count;
            }
        }

        assert!(!outer.is_empty(), "le pcap contient des tunnels CAPWAP");
        for (id, outer_count) in &outer {
            assert_eq!(
                inner.get(id),
                Some(outer_count),
                "tunnel {id} : la somme des paquets fils doit égaler le compteur du père"
            );
        }
        for id in inner.keys() {
            assert!(
                outer.contains_key(id),
                "tunnel {id} : lignes internes sans ligne externe"
            );
        }

        // Le graphe porte la parenté père/fils : chaque arête CAPWAP connaît
        // l'id de son tunnel, et des arêtes internes portent les ids des
        // tunnels qui les transportent (surbrillance au survol côté front).
        let capwap_edges: Vec<_> = graph
            .edges
            .values()
            .filter(|e| e.label == "CAPWAP")
            .collect();
        assert!(!capwap_edges.is_empty(), "arêtes CAPWAP attendues");
        assert!(
            capwap_edges.iter().all(|e| !e.encap_ids.is_empty()),
            "chaque arête CAPWAP doit porter au moins un id de tunnel"
        );
        let linked_inner = graph
            .edges
            .values()
            .filter(|e| e.label != "CAPWAP" && !e.encap_ids.is_empty())
            .count();
        assert!(
            linked_inner > 0,
            "des arêtes internes doivent être reliées à leur tunnel"
        );
    }
}
