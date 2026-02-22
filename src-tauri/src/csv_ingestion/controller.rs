use super::models;
use super::services;

#[tauri::command]
pub async fn process_csv_files(
	paths: Vec<String>,
) -> Result<models::ProcessCsvResult, String> {
	tauri::async_runtime::spawn_blocking(move || services::process_csv_files(paths))
		.await
		.map_err(|e| format!("Failed to execute CSV processing task: {e}"))?
}

#[tauri::command]
pub async fn merge_csv_group(paths: Vec<String>) -> Result<models::MergeCsvGroupResult, String> {
	tauri::async_runtime::spawn_blocking(move || services::merge_csv_group(paths))
		.await
		.map_err(|e| format!("Failed to execute CSV merge task: {e}"))?
}

#[tauri::command]
pub async fn create_table_from_csv_group(
	paths: Vec<String>,
	suggested_table_name: Option<String>,
) -> Result<models::CsvIngestionWriteResult, String> {
	tauri::async_runtime::spawn_blocking(move || {
		services::create_table_from_csv_group(paths, suggested_table_name)
	})
	.await
	.map_err(|e| format!("Failed to execute create-table task: {e}"))?
}

#[tauri::command]
pub async fn merge_csv_group_into_existing_table(
	paths: Vec<String>,
	table_name: String,
) -> Result<models::CsvIngestionWriteResult, String> {
	tauri::async_runtime::spawn_blocking(move || {
		services::merge_csv_group_into_existing_table(paths, table_name)
	})
	.await
	.map_err(|e| format!("Failed to execute merge-into-table task: {e}"))?
}