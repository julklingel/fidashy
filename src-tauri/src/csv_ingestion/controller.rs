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