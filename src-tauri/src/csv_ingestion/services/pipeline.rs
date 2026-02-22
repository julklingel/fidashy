use crate::csv_ingestion::models;
use crate::csv_ingestion::services::dedup;
use crate::csv_ingestion::services::grouping;
use crate::csv_ingestion::services::schema;
use rayon::prelude::*;

pub fn process_csv_files(paths: Vec<String>) -> Result<models::ProcessCsvResult, String> {
    if paths.is_empty() {
        return Err("No CSV file paths provided".to_string());
    }

    let files_with_schema: Vec<models::CsvFileSchemaInfo> = paths
        .into_par_iter()
        .map(schema::collect_file_schema)
        .collect::<Result<Vec<_>, _>>()?;

    // let dedup_report = dedup::DedupReport {
    //     in_batch_duplicates: dedup::detect_in_batch_duplicates(&files_with_schema),
    //     in_database_duplicates: dedup::detect_database_duplicates(&files_with_schema)?,
    // };

    // let _ = dedup_report.in_batch_duplicates.len() + dedup_report.in_database_duplicates.len();

    let processed_files = files_with_schema.len();
    let files = files_with_schema
        .iter()
        .map(|file| models::CsvFileHeaders {
            path: file.path.clone(),
            headers: file.headers.clone(),
        })
        .collect();
    let matching_header_groups = grouping::group_files_with_matching_headers(&files_with_schema);

    Ok(models::ProcessCsvResult {
        processed_files,
        files,
        matching_header_groups,
    })
}
