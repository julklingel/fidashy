use super::models;
use super::services;
use tauri::State;

#[tauri::command]
pub async fn process_csv_files(
	paths: Vec<String>,
	_state: State<'_, crate::db::models::DuckDbState>,
) -> Result<models::ProcessCsvResult, String> {
	tauri::async_runtime::spawn_blocking(move || services::process_csv_files(paths))
		.await
		.map_err(|e| format!("Failed to execute CSV processing task: {e}"))?
}

#[tauri::command]
pub async fn merge_csv_group(
	paths: Vec<String>,
	state: State<'_, crate::db::models::DuckDbState>,
) -> Result<models::MergeCsvGroupResult, String> {
	let db_state = state.inner().clone();
	tauri::async_runtime::spawn_blocking(move || services::merge_csv_group(paths, &db_state))
		.await
		.map_err(|e| format!("Failed to execute CSV merge task: {e}"))?
}

#[tauri::command]
pub async fn create_table_from_csv_group(
	paths: Vec<String>,
	suggested_table_name: Option<String>,
	state: State<'_, crate::db::models::DuckDbState>,
) -> Result<models::CsvIngestionWriteResult, String> {
	let db_state = state.inner().clone();
	tauri::async_runtime::spawn_blocking(move || {
		services::create_table_from_csv_group(paths, suggested_table_name, &db_state)
	})
	.await
	.map_err(|e| format!("Failed to execute create-table task: {e}"))?
}

#[tauri::command]
pub async fn merge_csv_group_into_existing_table(
	paths: Vec<String>,
	table_name: String,
	state: State<'_, crate::db::models::DuckDbState>,
) -> Result<models::CsvIngestionWriteResult, String> {
	let db_state = state.inner().clone();
	tauri::async_runtime::spawn_blocking(move || {
		services::merge_csv_group_into_existing_table(paths, table_name, &db_state)
	})
	.await
	.map_err(|e| format!("Failed to execute merge-into-table task: {e}"))?
}