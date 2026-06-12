use serde::Serialize;

#[derive(Serialize, Default, Debug)]
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
