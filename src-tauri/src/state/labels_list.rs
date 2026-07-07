use serde::Serialize;

#[derive(Serialize, Default, Debug)]

pub struct LabelStore {
    pub rows: Vec<(String, String, String)>,
}

impl LabelStore {
    pub fn new() -> Self {
        LabelStore { rows: Vec::new() }
    }

    pub fn add(&mut self, row: (String, String, String)) {
        self.rows.push(row)
    }

    pub fn get(&self) -> &Vec<(String, String, String)> {
        &self.rows
    }

    /// Fixe le label pour la clé `(mac, ip)` : met à jour la ligne existante,
    /// ou l'ajoute si absente. Utilisé par l'arbitrage des conflits.
    pub fn set(&mut self, mac: &str, ip: &str, label: &str) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.0 == mac && r.1 == ip) {
            row.2 = label.to_string();
        } else {
            self.rows
                .push((mac.to_string(), ip.to_string(), label.to_string()));
        }
    }

    pub fn clear(&mut self) {
        self.rows.clear()
    }
}

pub struct PcInfoLabel {
    pub label_lines: Vec<String>,
}

impl PcInfoLabel {
    pub fn new() -> Self {
        PcInfoLabel {
            label_lines: Vec::new(),
        }
    }

    pub fn get_label(&self) -> &Vec<String> {
        &self.label_lines
    }

    pub fn push(&mut self, label_line: String) {
        self.label_lines.push(label_line)
    }
}
