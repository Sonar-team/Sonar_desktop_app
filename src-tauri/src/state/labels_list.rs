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
