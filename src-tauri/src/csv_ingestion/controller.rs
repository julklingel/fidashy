use super::services;

// #[tauri::command]
// pub async fn group_csv_files(
// 	paths: Vec<String>,
// ) -> Result<models::ProcessCsvResult, String> {
// 	tauri::async_runtime::spawn_blocking(move || services::group_csv_files(paths))
// 		.await
// 		.map_err(|e| format!("Failed to execute CSV processing task: {e}"))?
// }


#[tauri::command]
pub async fn lazy_grouping_csv_many(
	paths: Vec<String>,
	) -> Result<Vec<Vec<String>>, String>  {
	tauri::async_runtime::spawn_blocking(move || {
		services::grouping::lazy_grouping_csv_many(paths)
	})
		.await
		.map_err(|e| format!("Failed to execute CSV processing task: {e}"))?
		.map_err(|e| format!("CSV grouping failed: {e}"))
}