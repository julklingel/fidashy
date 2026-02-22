use super::models;
use super::services;
use tauri::State;

#[tauri::command]
pub fn process_csv_files(
	paths: Vec<String>,
	_state: State<crate::db::models::DuckDbState>,
) -> Result<models::ProcessCsvResult, String> {
	services::process_csv_files(paths)
}

#[tauri::command]
pub fn merge_csv_group(
	paths: Vec<String>,
	state: State<crate::db::models::DuckDbState>,
) -> Result<models::MergeCsvGroupResult, String> {
	services::merge_csv_group(paths, &state)
}

#[tauri::command]
pub fn create_table_from_csv_group(
	paths: Vec<String>,
	suggested_table_name: Option<String>,
	state: State<crate::db::models::DuckDbState>,
) -> Result<models::CsvIngestionWriteResult, String> {
	services::create_table_from_csv_group(paths, suggested_table_name, &state)
}

#[tauri::command]
pub fn merge_csv_group_into_existing_table(
	paths: Vec<String>,
	table_name: String,
	state: State<crate::db::models::DuckDbState>,
) -> Result<models::CsvIngestionWriteResult, String> {
	services::merge_csv_group_into_existing_table(paths, table_name, &state)
}