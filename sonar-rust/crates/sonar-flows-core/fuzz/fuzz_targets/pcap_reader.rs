#![no_main]

use libfuzzer_sys::fuzz_target;
use sonar_flows_core::{matrix::FlowMatrix, pcap::append_pcap_file};
use std::path::PathBuf;

struct FuzzCapture(PathBuf);

impl FuzzCapture {
    fn for_current_process() -> Self {
        Self(std::env::temp_dir().join(format!("sonar-pcap-fuzz-{}.pcap", std::process::id())))
    }
}

impl Drop for FuzzCapture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// Fichier de travail propre au processus libFuzzer. Chaque entrée remplace
// son contenu avant de traverser libpcap puis `packet_parser`; les erreurs
// de format sont des résultats normaux, seuls panic/crash/UB intéressent le
// fuzzer.
fuzz_target!(|data: &[u8]| {
    let capture = FuzzCapture::for_current_process();
    if std::fs::write(&capture.0, data).is_err() {
        return;
    }

    let mut matrix = FlowMatrix::new();
    let _ = append_pcap_file(&mut matrix, &capture.0);
});
