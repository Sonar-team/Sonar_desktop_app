//! Test différentiel bout-en-bout PCAP -> matrice SFMS.
//!
//! La fixture est un échantillon déterministe de `The-Ultimate-PCAP` dont
//! l'inventaire indépendant TShark est committé à côté du PCAP. Le CSV de
//! référence fige ensuite chaque ligne produite (identité, compteurs, octets
//! et date), pas seulement les totaux globaux.

#![cfg(feature = "pcap")]
#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use packet_parser::LinkType;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sonar_flows_core::SonarCoreError;
use sonar_flows_core::csv::merge_matrix_files;
use sonar_flows_core::link::ethertype_from_name;
use sonar_flows_core::matrix::{FlowMatrix, FlowMatrixRow, parse_last_seen};
use sonar_flows_core::pcap::append_pcap_file;

const EXPECTED_PACKETS: usize = 328;
const EXPECTED_BYTES: u64 = 96_622;
const EXPECTED_ROWS: usize = 216;
const EXPECTED_COMMON_PACKETS: u64 = 313;
const EXPECTED_COMMON_BYTES: u64 = 94_091;
const EXPECTED_COMMON_FLOWS: usize = 202;

fn ultimate_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../src-tauri/test_files/pcaps/import/ultimate_ethernet_sample")
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../src-tauri/test_files/pcaps/import/pcap_tshark_corpus")
}

fn read_expected_rows(path: &Path) -> Vec<FlowMatrixRow> {
    let mut reader = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path(path)
        .expect("ouverture de la matrice oracle");
    reader
        .deserialize()
        .collect::<Result<Vec<_>, _>>()
        .expect("désérialisation de la matrice oracle")
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CommonFlowKey {
    mac_source: String,
    mac_destination: String,
    vlan_id: Option<u16>,
    ether_type: u16,
    ip_source: String,
    ip_destination: String,
    ip_protocol: Option<u8>,
    port_source: Option<u16>,
    port_destination: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommonFlowStats {
    count: u64,
    total_bytes: u64,
    last_seen_us: u64,
}

#[derive(Debug, Deserialize)]
struct RawCommonFlowRow {
    mac_source: String,
    mac_destination: String,
    vlan_id: String,
    ether_type: u16,
    ip_source: String,
    ip_destination: String,
    ip_protocol: String,
    port_source: String,
    port_destination: String,
    count: u64,
    total_bytes: u64,
    last_seen_us: u64,
}

fn transport_protocol_number(name: Option<&str>) -> Option<u8> {
    match name {
        Some("ICMP") => Some(1),
        Some("IGMP") => Some(2),
        Some("Tcp") => Some(6),
        Some("UDP") => Some(17),
        Some("IPv6") => Some(41),
        Some("GRE") => Some(47),
        Some("ESP") => Some(50),
        Some("ICMPv6") => Some(58),
        Some("EIGRP") => Some(88),
        Some("OSPFIGP") => Some(89),
        Some("L2TP") => Some(115),
        Some(other) => panic!("protocole de transport sans numéro oracle : {other}"),
        None => None,
    }
}

fn common_projection(rows: &[FlowMatrixRow]) -> BTreeMap<CommonFlowKey, CommonFlowStats> {
    let mut projected: BTreeMap<CommonFlowKey, CommonFlowStats> = BTreeMap::new();
    for row in rows {
        let ether_type = ethertype_from_name(&row.protocol_data_link)
            .unwrap_or_else(|| panic!("EtherType SFMS inconnu : {}", row.protocol_data_link))
            .0;
        // Même domaine que l'oracle : les longueurs IEEE 802.3 et les VLAN
        // empilés ne sont pas représentables sans perte dans SFMS v1.
        if ether_type < 0x0600 {
            continue;
        }
        let key = CommonFlowKey {
            mac_source: row.mac_source.clone(),
            mac_destination: row.mac_destination.clone(),
            vlan_id: row.vlan_id,
            ether_type,
            ip_source: row.ip_source.clone(),
            ip_destination: row.ip_destination.clone(),
            ip_protocol: transport_protocol_number(row.protocol_transport.as_deref()),
            port_source: row.port_source,
            port_destination: row.port_destination,
        };
        let last_seen_us = parse_last_seen(&row.last_seen)
            .expect("timestamp SFMS valide")
            .duration_since(UNIX_EPOCH)
            .expect("timestamp postérieur à epoch")
            .as_micros() as u64;
        let stats = projected.entry(key).or_insert(CommonFlowStats {
            count: 0,
            total_bytes: 0,
            last_seen_us: 0,
        });
        stats.count += row.count;
        stats.total_bytes += row.total_bytes;
        stats.last_seen_us = stats.last_seen_us.max(last_seen_us);
    }
    projected
}

fn read_tshark_common_flows(path: &Path) -> BTreeMap<CommonFlowKey, CommonFlowStats> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .from_path(path)
        .expect("ouverture de l'oracle TShark");
    reader
        .deserialize::<RawCommonFlowRow>()
        .map(|row| {
            let row = row.expect("ligne TShark commune valide");
            (
                CommonFlowKey {
                    mac_source: row.mac_source,
                    mac_destination: row.mac_destination,
                    vlan_id: (!row.vlan_id.is_empty())
                        .then(|| row.vlan_id.parse().expect("VLAN TShark valide")),
                    ether_type: row.ether_type,
                    ip_source: row.ip_source,
                    ip_destination: row.ip_destination,
                    ip_protocol: (!row.ip_protocol.is_empty())
                        .then(|| row.ip_protocol.parse().expect("protocole IP TShark valide")),
                    port_source: (!row.port_source.is_empty())
                        .then(|| row.port_source.parse().expect("port source TShark valide")),
                    port_destination: (!row.port_destination.is_empty()).then(|| {
                        row.port_destination
                            .parse()
                            .expect("port destination TShark valide")
                    }),
                },
                CommonFlowStats {
                    count: row.count,
                    total_bytes: row.total_bytes,
                    last_seen_us: row.last_seen_us,
                },
            )
        })
        .collect()
}

#[derive(Debug)]
struct OracleMetadata {
    capture_sha256: String,
    included_packets: u64,
    included_bytes: u64,
}

fn read_oracle_metadata(path: &Path) -> OracleMetadata {
    let content = std::fs::read_to_string(path).expect("lecture des métadonnées oracle");
    let value = |name: &str| {
        content
            .lines()
            .find_map(|line| line.strip_prefix(&format!("# {name}=")))
            .unwrap_or_else(|| panic!("métadonnée {name} absente de {}", path.display()))
    };
    OracleMetadata {
        capture_sha256: value("capture_sha256").to_string(),
        included_packets: value("included_packets")
            .parse()
            .expect("compte de paquets oracle"),
        included_bytes: value("included_bytes")
            .parse()
            .expect("compte d'octets oracle"),
    }
}

fn assert_oracle_hash(capture: &Path, oracle: &Path) {
    let expected = read_oracle_metadata(oracle).capture_sha256;
    let actual = format!(
        "{:x}",
        Sha256::digest(std::fs::read(capture).expect("lecture de la capture"))
    );
    assert_eq!(
        actual,
        expected,
        "oracle TShark périmé pour {} : régénérer {}",
        capture.display(),
        oracle.display()
    );
}

fn assert_common_flows_equal(
    label: &str,
    actual: &BTreeMap<CommonFlowKey, CommonFlowStats>,
    expected: &BTreeMap<CommonFlowKey, CommonFlowStats>,
) {
    if actual == expected {
        return;
    }
    let keys: BTreeSet<_> = actual.keys().chain(expected.keys()).collect();
    let differences = keys
        .into_iter()
        .filter_map(|key| {
            let actual = actual.get(key);
            let expected = expected.get(key);
            (actual != expected)
                .then(|| format!("{key:?}\n  SONAR={actual:?}\n  TShark={expected:?}"))
        })
        .take(20)
        .collect::<Vec<_>>()
        .join("\n");
    panic!("écart SONAR/TShark pour {label} :\n{differences}");
}

fn assert_deterministic_round_trip(matrix: &FlowMatrix, label: &str) {
    let base = std::env::temp_dir().join(format!("sonar_issue168_{label}_{}", std::process::id()));
    let first = base.with_extension("first.csv");
    let second = base.with_extension("second.csv");
    matrix
        .export_to_csv(first.to_string_lossy().into_owned())
        .expect("premier export");
    matrix
        .export_to_csv(second.to_string_lossy().into_owned())
        .expect("second export");
    assert_eq!(
        std::fs::read(&first).expect("lecture premier export"),
        std::fs::read(&second).expect("lecture second export"),
        "export non déterministe pour {label}"
    );

    let reimported = merge_matrix_files(std::slice::from_ref(&first)).expect("réimport SFMS");
    let mut actual = reimported.to_flat_vec();
    let mut expected = matrix.to_flat_vec();
    for row in actual.iter_mut().chain(expected.iter_mut()) {
        row.origin.clear();
    }
    assert_eq!(
        actual, expected,
        "aller-retour SFMS avec perte pour {label}"
    );
    let _ = std::fs::remove_file(first);
    let _ = std::fs::remove_file(second);
}

#[test]
fn ultimate_ethernet_sample_matches_sfms_snapshot_flow_by_flow() {
    let fixtures = ultimate_fixture_dir();
    let capture = fixtures.join("ultimate_ethernet_sample.pcap");
    let expected_csv = fixtures.join("ultimate_ethernet_sample.csv");

    let mut matrix = FlowMatrix::new();
    let report = append_pcap_file(&mut matrix, &capture).expect("conversion de la fixture");

    // Ces deux agrégats viennent directement de l'inventaire TShark committé
    // dans ultimate_ethernet_sample.tshark.md.
    assert_eq!(report.packets, EXPECTED_PACKETS);
    assert_eq!(report.parse_ok, EXPECTED_PACKETS);
    assert_eq!(report.parse_errors, 0);
    assert_eq!(matrix.link_type, Some(LinkType::ETHERNET));

    let actual = matrix.to_flat_vec();
    let expected = read_expected_rows(&expected_csv);
    assert_eq!(actual.len(), EXPECTED_ROWS);
    assert_eq!(actual, expected, "écart de matrice flux par flux");

    assert_eq!(
        actual.iter().map(|row| row.count).sum::<u64>(),
        EXPECTED_PACKETS as u64,
        "la somme des compteurs doit rester égale aux trames TShark"
    );
    assert_eq!(
        actual.iter().map(|row| row.total_bytes).sum::<u64>(),
        EXPECTED_BYTES,
        "la somme des octets doit rester égale à frame.len dans TShark"
    );
}

#[test]
fn ultimate_ethernet_sample_matches_tshark_on_every_shared_field() {
    let fixtures = ultimate_fixture_dir();
    let capture = fixtures.join("ultimate_ethernet_sample.pcap");
    let oracle = fixtures.join("ultimate_ethernet_sample.flows.tsv");
    assert_oracle_hash(&capture, &oracle);

    let mut matrix = FlowMatrix::new();
    append_pcap_file(&mut matrix, &capture).expect("conversion de la fixture");

    let actual = common_projection(&matrix.to_flat_vec());
    let expected = read_tshark_common_flows(&oracle);
    assert_eq!(expected.len(), EXPECTED_COMMON_FLOWS);
    assert_eq!(
        expected.values().map(|stats| stats.count).sum::<u64>(),
        EXPECTED_COMMON_PACKETS
    );
    assert_eq!(
        expected
            .values()
            .map(|stats| stats.total_bytes)
            .sum::<u64>(),
        EXPECTED_COMMON_BYTES
    );
    assert_common_flows_equal("ultimate_ethernet_sample", &actual, &expected);
}

#[test]
fn ultimate_ethernet_sample_export_is_byte_for_byte_deterministic() {
    let fixtures = ultimate_fixture_dir();
    let capture = fixtures.join("ultimate_ethernet_sample.pcap");
    let expected_csv = fixtures.join("ultimate_ethernet_sample.csv");
    let output = std::env::temp_dir().join(format!(
        "sonar_ultimate_ethernet_sample_{}.csv",
        std::process::id()
    ));

    let mut matrix = FlowMatrix::new();
    append_pcap_file(&mut matrix, &capture).expect("conversion de la fixture");
    matrix
        .export_to_csv(output.to_string_lossy().into_owned())
        .expect("export déterministe");

    let actual = std::fs::read(&output).expect("lecture de l'export");
    let expected = std::fs::read(&expected_csv).expect("lecture du snapshot");
    let _ = std::fs::remove_file(&output);
    assert_eq!(actual, expected, "le CSV SFMS a dérivé octet par octet");
}

#[test]
fn multi_dlt_corpus_matches_tshark_flow_by_flow() {
    let fixtures = corpus_dir();
    for (name, expected_link_type) in [
        ("linux_sll", LinkType::LINUX_SLL),
        ("linux_sll2", LinkType::LINUX_SLL2),
        ("raw_ip", LinkType::RAW),
        ("vlan", LinkType::ETHERNET),
        ("industrial_ethernet", LinkType::ETHERNET),
        ("capwap_radius", LinkType::ETHERNET),
        ("capwap_management", LinkType::ETHERNET),
    ] {
        let capture = fixtures.join(format!("{name}.pcap"));
        let oracle = fixtures.join(format!("{name}.flows.tsv"));
        assert_oracle_hash(&capture, &oracle);
        let metadata = read_oracle_metadata(&oracle);

        let mut matrix = FlowMatrix::new();
        let report = append_pcap_file(&mut matrix, &capture)
            .unwrap_or_else(|error| panic!("conversion de {name}: {error}"));
        assert_eq!(report.packets as u64, metadata.included_packets, "{name}");
        assert_eq!(report.parse_ok, report.packets, "{name}");
        assert_eq!(report.parse_errors, 0, "{name}");
        assert_eq!(matrix.link_type, Some(expected_link_type), "{name}");

        let actual = common_projection(&matrix.to_flat_vec());
        let expected = read_tshark_common_flows(&oracle);
        assert_eq!(
            expected
                .values()
                .map(|stats| stats.total_bytes)
                .sum::<u64>(),
            metadata.included_bytes,
            "métadonnées d'oracle incohérentes pour {name}"
        );
        assert_common_flows_equal(name, &actual, &expected);
        assert_deterministic_round_trip(&matrix, name);
    }
}

#[test]
fn capwap_data_matches_outer_and_inner_tshark_flows_and_encap_id() {
    let fixtures = corpus_dir();
    let capture = fixtures.join("capwap_data.pcap");
    let outer_oracle = fixtures.join("capwap_data.flows.tsv");
    let inner_oracle = fixtures.join("capwap_data.inner.flows.tsv");
    assert_oracle_hash(&capture, &outer_oracle);
    assert_oracle_hash(&capture, &inner_oracle);

    let mut matrix = FlowMatrix::new();
    let report = append_pcap_file(&mut matrix, &capture).expect("conversion CAPWAP");
    assert_eq!(
        (report.packets, report.parse_ok, report.parse_errors),
        (2, 2, 0)
    );

    let rows = matrix.to_flat_vec();
    assert_eq!(rows.len(), 4, "deux niveaux par trame CAPWAP");
    let encap_ids = rows
        .iter()
        .map(|row| {
            assert!(!row.encap_id.is_empty(), "encap_id CAPWAP absent");
            row.encap_id.as_str()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        encap_ids.len(),
        1,
        "les deux sens et niveaux sont joignables"
    );
    assert!(
        encap_ids.iter().all(|id| id.len() == 16),
        "encap_id hexadécimal stable sur 64 bits"
    );

    let outer_rows = rows
        .iter()
        .filter(|row| row.application_protocol.as_deref() == Some("CAPWAP"))
        .cloned()
        .collect::<Vec<_>>();
    let inner_rows = rows
        .iter()
        .filter(|row| row.application_protocol.as_deref() != Some("CAPWAP"))
        .cloned()
        .collect::<Vec<_>>();
    assert_common_flows_equal(
        "capwap_data/outer",
        &common_projection(&outer_rows),
        &read_tshark_common_flows(&outer_oracle),
    );
    assert_common_flows_equal(
        "capwap_data/inner",
        &common_projection(&inner_rows),
        &read_tshark_common_flows(&inner_oracle),
    );
    assert_deterministic_round_trip(&matrix, "capwap_data");
}

#[test]
fn unsupported_ieee80211_capture_is_refused_before_matrix_mutation() {
    let fixtures = corpus_dir();
    let capture = fixtures.join("unsupported_ieee80211.pcapng");
    assert_oracle_hash(&capture, &fixtures.join("unsupported_ieee80211.flows.tsv"));
    let mut matrix = FlowMatrix::new();
    let error = append_pcap_file(&mut matrix, &capture).expect_err("DLT 802.11 refusé");
    assert!(matches!(error, SonarCoreError::UnsupportedLinkType { .. }));
    assert_eq!(matrix.row_count(), 0);
}
