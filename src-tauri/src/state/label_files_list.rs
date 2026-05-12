use serde::Serialize;

#[derive(Serialize, Default, Debug)]
pub struct SelectedLabelFiles {
    pub files_names: Vec<String>,
}

impl SelectedLabelFiles {

    pub fn new() -> Self {
        SelectedLabelFiles { files_names : Vec::new() }
    }

    pub fn set (&mut self, files_names: Vec<String>) {

        self.files_names = files_names;
    }

    pub fn get (&self) -> &Vec<String> {
        &self.files_names
    }
}