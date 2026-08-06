#![no_main]

use libfuzzer_sys::fuzz_target;
use sonar_flows_core::csv::read_matrix_rows;
use std::path::PathBuf;

struct FuzzMatrix(PathBuf);

impl FuzzMatrix {
    fn for_current_process() -> Self {
        Self(std::env::temp_dir().join(format!("sonar-matrix-fuzz-{}.csv", std::process::id())))
    }
}

impl Drop for FuzzMatrix {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// Chemin d'import des matrices SFMS : préambule `#SFMS` (versions, DLT,
// contexte percent-encodé) puis lignes CSV validées champ par champ. Un
// fichier hostile doit produire une erreur typée, jamais un panic, une boucle
// ou une ligne acceptée à tort — seuls panic/crash/UB intéressent le fuzzer.
fuzz_target!(|data: &[u8]| {
    let matrix = FuzzMatrix::for_current_process();
    if std::fs::write(&matrix.0, data).is_err() {
        return;
    }

    let _ = read_matrix_rows(&matrix.0);
});
