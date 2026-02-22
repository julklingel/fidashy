use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CsvFileHeaders {
    pub path: String,
    pub headers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessCsvResult {
    pub processed_files: usize,
    pub files: Vec<CsvFileHeaders>,
}


