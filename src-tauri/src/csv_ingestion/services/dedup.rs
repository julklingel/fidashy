use crate::csv_ingestion::models;


#[derive(Debug, Clone, Default)]
pub struct DedupReport {
    pub in_batch_duplicates: Vec<String>,
    pub in_database_duplicates: Vec<String>,
}

pub fn detect_in_batch_duplicates(files: &[models::CsvFileSchemaInfo]) -> &str {
    return "TBD"
}

pub fn detect_database_duplicates(_files: &[models::CsvFileSchemaInfo]) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}
