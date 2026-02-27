use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CsvFileHeaders {
    pub path: String,
    pub headers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CsvFileSchemaInfo {
    pub path: String,
    pub headers: Vec<String>,
    pub schema_signature_hash: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchingHeaderGroup {
    pub headers: Vec<String>,
    pub file_paths: Vec<String>,
    pub duplicate_rows: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessCsvResult {
    pub processed_files: usize,
    pub group_count: usize,
    pub total_duplicate_rows: usize,
    pub files: Vec<CsvFileHeaders>,
    pub matching_header_groups: Vec<MatchingHeaderGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupWithDuplicates {
    pub paths: Vec<String>,
    pub duplicate_count: usize,
    pub total_entries: usize,
}
