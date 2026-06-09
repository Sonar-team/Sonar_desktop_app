use serde::Serialize;

#[derive(Serialize, Default, Debug)]
pub struct SelectedLabelFiles {
    pub files_names: Vec<String>,
}

impl SelectedLabelFiles {

    pub fn new() -> Self {
        SelectedLabelFiles { files_names : Vec::new() }
    }

    /*pub fn set (&mut self, files_names: Vec<String>) {

        self.files_names = files_names;
    }*/

    pub fn get (&self) -> &Vec<String> {
        &self.files_names
    }

    pub fn add(&mut self, file_name: String) {
        self.files_names.push(file_name);
        self.files_names.sort()
    }

    pub fn remove(&mut self, file_name: &str) {
        self.files_names.retain(|f| f != file_name)
    }
}

#[derive(Serialize, Default, Debug)]
pub struct PcInfoLabel {
    pub label_lines: Vec<String>
}

impl PcInfoLabel {

    pub fn new() -> Self {
        PcInfoLabel { label_lines: Vec::new() } 
    }

    pub fn get_label (&self) -> &Vec<String> {
        &self.label_lines
    }

    pub fn push (&mut self, label_line : String) {
        self.label_lines.push(label_line)
    }


}