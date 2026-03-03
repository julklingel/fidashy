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
pub async fn deduplicate_cached_group(
	group_id: String,
	cache: tauri::State<'_, models::MergeCache>,
) -> Result<models::DeduplicateGroupResult, String> {
	let cache = cache.inner().clone();
	let group_id_for_error = group_id.clone();
	tauri::async_runtime::spawn_blocking(move || {
		services::merge_groups::deduplicate_cached_group(&group_id, &cache)
	})
		.await
		.map_err(|e| {
			format!(
				"Failed to execute deduplication task for group '{}': {e}",
				group_id_for_error
			)
		})?
		.map_err(|e| format!("Deduplication failed for group '{}': {e}", group_id_for_error))
}