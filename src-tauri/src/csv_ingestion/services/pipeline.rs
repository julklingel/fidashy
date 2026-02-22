use crate::csv_ingestion::models;
use crate::csv_ingestion::services::dedup;
use crate::csv_ingestion::services::grouping;
use crate::csv_ingestion::services::schema;
use rayon::prelude::*;

fn derive_merged_headers(files: &[models::CsvFileHeaders]) -> Vec<String> {
    files
        .first()
        .map(|file| file.headers.clone())
        .unwrap_or_default()
}

pub fn process_csv_files(paths: Vec<String>) -> Result<models::ProcessCsvResult, String> {
    if paths.is_empty() {
        return Err("No CSV file paths provided".to_string());
    }

    let files_with_schema: Vec<models::CsvFileSchemaInfo> = paths
        .into_par_iter()
        .map(schema::collect_file_schema)
        .collect::<Result<Vec<_>, _>>()?;

    let processed_files = files_with_schema.len();
    let files = files_with_schema
        .iter()
        .map(|file| models::CsvFileHeaders {
            path: file.path.clone(),
            headers: file.headers.clone(),
        })
        .collect();
    let grouped = grouping::group_files_with_matching_headers(&files_with_schema);
    let matching_header_groups = dedup::annotate_group_duplicates(grouped)?;

    let group_count = matching_header_groups.len();
    let total_duplicate_rows = matching_header_groups
        .iter()
        .map(|group| group.duplicate_rows)
        .sum();

    Ok(models::ProcessCsvResult {
        processed_files,
        group_count,
        total_duplicate_rows,
        files,
        matching_header_groups,
    })
}

pub fn merge_csv_group(paths: Vec<String>) -> Result<models::MergeCsvGroupResult, String> {
    let process_result = process_csv_files(paths)?;
    let merged_headers = derive_merged_headers(&process_result.files);

    Ok(models::MergeCsvGroupResult {
        input_rows: 0,
        merged_rows: 0,
        duplicate_rows_removed: process_result.total_duplicate_rows,
        merged_columns: merged_headers.len(),
        merged_headers,
        matching_table_name: None,
        duplicate_rows_with_db: 0,
        requires_user_choice: false,
    })
}

pub fn create_table_from_csv_group(
    paths: Vec<String>,
    suggested_table_name: Option<String>,
) -> Result<models::CsvIngestionWriteResult, String> {
    let process_result = process_csv_files(paths)?;
    let table_name = suggested_table_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("csv_group_refactor_placeholder")
        .to_string();

    Ok(models::CsvIngestionWriteResult {
        table_name,
        input_rows: 0,
        rows_inserted: 0,
        rows_skipped_duplicates: process_result.total_duplicate_rows,
        created_new_table: true,
    })
}

pub fn merge_csv_group_into_existing_table(
    paths: Vec<String>,
    table_name: String,
) -> Result<models::CsvIngestionWriteResult, String> {
    if table_name.trim().is_empty() {
        return Err("A target table name is required for merge".to_string());
    }

    let process_result = process_csv_files(paths)?;

    Ok(models::CsvIngestionWriteResult {
        table_name,
        input_rows: 0,
        rows_inserted: 0,
        rows_skipped_duplicates: process_result.total_duplicate_rows,
        created_new_table: false,
    })
}
