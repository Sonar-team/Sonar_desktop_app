//! Tests d'intégration de `sonar-cli` : le binaire est lancé comme un vrai
//! utilisateur (arguments, fichiers, codes de sortie, stderr), sans passer
//! par les API internes. Les PCAP d'entrée sont forgés octet par octet pour
//! rester déterministes et ne dépendre d'aucune fixture binaire.

// Même politique que allow-unwrap-in-tests/allow-expect-in-tests (clippy.toml),
// que clippy n'applique pas aux tests d'intégration.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Chemin du binaire compilé par Cargo pour ces tests.
fn sonar_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sonar-cli"))
}

/// Répertoire temporaire propre, isolé par nom de test.
fn temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sonar_cli_it_{test_name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("création du répertoire temporaire");
    dir
}

/// Fixture CSV de référence partagée avec les tests de `sonar-flows-core`.
fn matrix_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../src-tauri/test_files")
        .join("20260703_NP_Matrice.csv")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ---------------------------------------------------------------------------
// Forge PCAP : en-tête global + trames Ethernet/IPv4/UDP minimales valides.
// ---------------------------------------------------------------------------

/// En-tête global PCAP (little-endian, link-type Ethernet).
fn pcap_global_header() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes()); // magic
    bytes.extend_from_slice(&2u16.to_le_bytes()); // version majeure
    bytes.extend_from_slice(&4u16.to_le_bytes()); // version mineure
    bytes.extend_from_slice(&0i32.to_le_bytes()); // thiszone
    bytes.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
    bytes.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    bytes.extend_from_slice(&1u32.to_le_bytes()); // LINKTYPE_ETHERNET
    bytes
}

/// Trame Ethernet II + IPv4 + UDP (checksum IP correcte, UDP sans checksum).
fn udp_frame(src_last_byte: u8, dst_last_byte: u8) -> Vec<u8> {
    let mut frame = Vec::new();
    // Ethernet : adresses localement administrées, ethertype IPv4.
    frame.extend_from_slice(&[0x02, 0, 0, 0, 0, dst_last_byte]);
    frame.extend_from_slice(&[0x02, 0, 0, 0, 0, src_last_byte]);
    frame.extend_from_slice(&[0x08, 0x00]);

    // IPv4 : 20 octets d'en-tête + 12 d'UDP, protocole 17, TTL 64.
    let mut ip = Vec::new();
    ip.extend_from_slice(&[0x45, 0x00, 0x00, 0x20]); // version/IHL, TOS, longueur totale = 32
    ip.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // identification, flags/fragment
    ip.extend_from_slice(&[0x40, 0x11, 0x00, 0x00]); // TTL, protocole UDP, checksum (calculée après)
    ip.extend_from_slice(&[192, 168, 1, src_last_byte]); // IP source
    ip.extend_from_slice(&[192, 168, 1, dst_last_byte]); // IP destination
    let checksum = ipv4_checksum(&ip);
    ip[10..12].copy_from_slice(&checksum.to_be_bytes());
    frame.extend_from_slice(&ip);

    // UDP : 1234 -> 53, longueur 12 (8 + payload "test"), checksum optionnelle.
    frame.extend_from_slice(&[0x04, 0xd2, 0x00, 0x35, 0x00, 0x0c, 0x00, 0x00]);
    frame.extend_from_slice(b"test");
    frame
}

fn ipv4_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for word in header.chunks(2) {
        sum += u32::from(u16::from_be_bytes([word[0], word[1]]));
    }
    while sum > 0xffff {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Ajoute un enregistrement PCAP (en-tête + données) au fichier en cours.
fn push_record(bytes: &mut Vec<u8>, ts_sec: u32, frame: &[u8]) {
    bytes.extend_from_slice(&ts_sec.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
    bytes.extend_from_slice(&(frame.len() as u32).to_le_bytes()); // incl_len
    bytes.extend_from_slice(&(frame.len() as u32).to_le_bytes()); // orig_len
    bytes.extend_from_slice(frame);
}

/// Écrit un PCAP de `frames` trames, une par seconde.
fn write_pcap(path: &Path, frames: &[Vec<u8>]) {
    let mut bytes = pcap_global_header();
    for (i, frame) in frames.iter().enumerate() {
        push_record(&mut bytes, i as u32, frame);
    }
    std::fs::write(path, bytes).expect("écriture du pcap forgé");
}

// ---------------------------------------------------------------------------
// Lecture de la matrice CSV produite (via les en-têtes, pas les positions).
// ---------------------------------------------------------------------------

/// Ouvre une matrice exportée en sautant la ligne de préambule `#SFMS`.
fn matrix_csv_reader(path: &Path) -> csv::Reader<std::fs::File> {
    csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path(path)
        .expect("ouverture du CSV exporté")
}

/// Somme la colonne `count` de la matrice exportée.
fn sum_count_column(path: &Path) -> u64 {
    let mut reader = matrix_csv_reader(path);
    let count_idx = reader
        .headers()
        .expect("en-têtes CSV")
        .iter()
        .position(|h| h == "count")
        .expect("colonne count");
    reader
        .records()
        .map(|record| {
            record.expect("ligne CSV")[count_idx]
                .parse::<u64>()
                .expect("count numérique")
        })
        .sum()
}

fn data_row_count(path: &Path) -> usize {
    let mut reader = matrix_csv_reader(path);
    reader.records().count()
}

// ---------------------------------------------------------------------------
// Commande `pcap`
// ---------------------------------------------------------------------------

#[test]
fn pcap_converts_forged_capture_to_matrix_csv() {
    let dir = temp_dir("pcap_ok");
    let input = dir.join("capture.pcap");
    let output = dir.join("matrice.csv");
    // Deux paquets du même flux + un paquet du flux retour.
    write_pcap(&input, &[udp_frame(1, 2), udp_frame(1, 2), udp_frame(2, 1)]);

    let result = sonar_cli()
        .arg("pcap")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(result.status.success(), "exit 0 attendu, stderr: {stderr}");
    assert!(
        stderr.contains("3 paquet(s) lus"),
        "le bilan par fichier doit apparaître sur stderr: {stderr}"
    );
    assert_eq!(
        sum_count_column(&output),
        3,
        "les 3 paquets doivent être comptés dans la matrice"
    );
    assert_eq!(
        data_row_count(&output),
        2,
        "aller et retour = 2 flux distincts"
    );
}

#[test]
fn pcap_accumulates_multiple_input_files() {
    let dir = temp_dir("pcap_multi");
    let input_a = dir.join("site-a.pcap");
    let input_b = dir.join("site-b.pcap");
    let output = dir.join("matrice.csv");
    write_pcap(&input_a, &[udp_frame(1, 2)]);
    write_pcap(&input_b, &[udp_frame(1, 2), udp_frame(1, 2)]);

    let result = sonar_cli()
        .arg("pcap")
        .arg(&input_a)
        .arg(&input_b)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(result.status.success(), "exit 0 attendu, stderr: {stderr}");
    assert_eq!(
        sum_count_column(&output),
        3,
        "les compteurs des deux fichiers doivent se cumuler"
    );
    assert_eq!(data_row_count(&output), 1, "même flux -> une seule ligne");
}

#[test]
fn pcap_fails_on_truncated_file_without_writing_output() {
    let dir = temp_dir("pcap_truncated");
    let input = dir.join("truncated.pcap");
    let output = dir.join("matrice.csv");
    // En-tête d'enregistrement annonçant 100 octets, 10 présents : l'erreur
    // de lecture doit faire échouer la conversion (pas une matrice partielle).
    let mut bytes = pcap_global_header();
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&100u32.to_le_bytes());
    bytes.extend_from_slice(&100u32.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 10]);
    std::fs::write(&input, bytes).expect("écriture du pcap tronqué");

    let result = sonar_cli()
        .arg("pcap")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(!result.status.success(), "exit 1 attendu, stderr: {stderr}");
    assert!(
        stderr.contains("error:") && stderr.contains("truncated.pcap"),
        "l'erreur doit citer le fichier fautif: {stderr}"
    );
    assert!(
        !output.exists(),
        "aucun CSV ne doit être écrit quand la conversion échoue"
    );
}

#[test]
fn pcap_fails_on_missing_input_file() {
    let dir = temp_dir("pcap_missing");
    let output = dir.join("matrice.csv");

    let result = sonar_cli()
        .arg("pcap")
        .arg(dir.join("absent.pcap"))
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(!result.status.success(), "exit 1 attendu, stderr: {stderr}");
    assert!(
        stderr.contains("error:") && stderr.contains("absent.pcap"),
        "l'erreur doit citer le fichier manquant: {stderr}"
    );
    assert!(!output.exists(), "aucun CSV ne doit être écrit");
}

// ---------------------------------------------------------------------------
// Commande `matrix`
// ---------------------------------------------------------------------------

#[test]
fn matrix_merges_two_files_and_doubles_counters() {
    let dir = temp_dir("matrix_merge");
    let file_a = dir.join("site-a.csv");
    let file_b = dir.join("site-b.csv");
    let single_out = dir.join("simple.csv");
    let merged_out = dir.join("fusion.csv");
    std::fs::copy(matrix_fixture(), &file_a).expect("copie a");
    std::fs::copy(matrix_fixture(), &file_b).expect("copie b");

    let single = sonar_cli()
        .arg("matrix")
        .arg(&file_a)
        .arg("--output")
        .arg(&single_out)
        .output()
        .expect("lancement de sonar-cli (simple)");
    assert!(single.status.success(), "stderr: {}", stderr_of(&single));

    let merged = sonar_cli()
        .arg("matrix")
        .arg(&file_a)
        .arg(&file_b)
        .arg("--output")
        .arg(&merged_out)
        .output()
        .expect("lancement de sonar-cli (fusion)");
    assert!(merged.status.success(), "stderr: {}", stderr_of(&merged));

    assert_eq!(
        data_row_count(&merged_out),
        data_row_count(&single_out),
        "mêmes flux -> même nombre de lignes"
    );
    assert_eq!(
        sum_count_column(&merged_out),
        sum_count_column(&single_out) * 2,
        "compteurs cumulés à la fusion"
    );
}

#[test]
fn matrix_fails_on_invalid_csv_and_reports_line_number() {
    let dir = temp_dir("matrix_invalid");
    let bad = dir.join("bad.csv");
    let output = dir.join("fusion.csv");
    std::fs::write(&bad, "mac_source,mac_destination\nnot,enough,columns\n")
        .expect("écriture du CSV invalide");

    let result = sonar_cli()
        .arg("matrix")
        .arg(&bad)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(!result.status.success(), "exit 1 attendu, stderr: {stderr}");
    assert!(
        stderr.contains("error:") && stderr.contains("ligne 2"),
        "l'erreur doit indiquer la ligne fautive: {stderr}"
    );
    assert!(!output.exists(), "aucun CSV ne doit être écrit");
}

// ---------------------------------------------------------------------------
// Commande `graph`
// ---------------------------------------------------------------------------

#[test]
fn graph_exports_filtered_dot_with_ip_labels() {
    let dir = temp_dir("graph_dot");
    let pcap = dir.join("capture.pcap");
    let matrix = dir.join("matrix.csv");
    let graph = dir.join("network.dot");
    write_pcap(&pcap, &[udp_frame(1, 2), udp_frame(3, 4)]);

    let conversion = sonar_cli()
        .arg("pcap")
        .arg(&pcap)
        .arg("--output")
        .arg(&matrix)
        .output()
        .expect("conversion PCAP");
    assert!(
        conversion.status.success(),
        "stderr: {}",
        stderr_of(&conversion)
    );

    let result = sonar_cli()
        .arg("graph")
        .arg(&matrix)
        .arg("--output")
        .arg(&graph)
        .arg("--min-bytes")
        .arg("40")
        .arg("--max-nodes")
        .arg("2")
        .arg("--protocol")
        .arg("udp")
        .arg("--labels")
        .arg("ip")
        .output()
        .expect("export DOT");

    let stderr = stderr_of(&result);
    assert!(result.status.success(), "exit 0 attendu, stderr: {stderr}");
    let dot = std::fs::read_to_string(&graph).expect("lecture DOT");
    assert!(dot.starts_with("digraph sonar"));
    assert_eq!(dot.matches("fillcolor=").count(), 2, "plafond de nœuds");
    assert_eq!(dot.matches(" -> ").count(), 1, "une relation conservée");
    assert!(dot.contains("label=\"192.168.1."), "labels IP attendus");
    assert!(dot.contains("UDP\\n53\\n46 o"), "protocole, port et volume");
}

#[test]
fn graph_rejects_unknown_output_format() {
    let dir = temp_dir("graph_format");
    let output = dir.join("network.jpg");

    let result = sonar_cli()
        .arg("graph")
        .arg(matrix_fixture())
        .arg("--output")
        .arg(&output)
        .output()
        .expect("lancement de sonar-cli");

    let stderr = stderr_of(&result);
    assert!(!result.status.success(), "exit 1 attendu, stderr: {stderr}");
    assert!(stderr.contains(".dot, .svg ou .png"));
    assert!(!output.exists());
}
