use super::{models, services};


#[tauri::command]
pub async fn lazy_grouping_csv_many(
	paths: Vec<String>,
	cache: tauri::State<'_, models::MergeCache>,
	) -> Result<Vec<models::GroupWithDuplicates>, String>  {
	let cache = cache.inner().clone();
	tauri::async_runtime::spawn_blocking(move || {
		services::grouping::lazy_grouping_csv_many(paths, &cache)
	})
		.await
		.map_err(|e| format!("Failed to execute CSV processing task: {e}"))?
		.map_err(|e| format!("CSV grouping failed: {e}"))
}

#[tauri::command]
pub async fn merge_csv(
	group_id: String,
	cache: tauri::State<'_, models::MergeCache>,
) -> Result<String, String> {
	let cache = cache.inner().clone();
	tauri::async_runtime::spawn_blocking(move || services::merge_groups::merge_csv(&group_id, &cache))
		.await
		.map_err(|e| format!("Failed to execute merge task: {e}"))?
}