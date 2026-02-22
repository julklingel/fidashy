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
pub struct MergeCsvGroupResult {
    pub input_rows: usize,
    pub merged_rows: usize,
    pub duplicate_rows_removed: usize,
    pub merged_columns: usize,
    pub merged_headers: Vec<String>,
    pub matching_table_name: Option<String>,
    pub duplicate_rows_with_db: usize,
    pub requires_user_choice: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CsvIngestionWriteResult {
    pub table_name: String,
    pub input_rows: usize,
    pub rows_inserted: usize,
    pub rows_skipped_duplicates: usize,
    pub created_new_table: bool,
}

