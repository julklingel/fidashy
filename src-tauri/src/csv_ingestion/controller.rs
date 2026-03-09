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

#[tauri::command]
pub async fn skip_merge_cached_group(
	group_id: String,
	cache: tauri::State<'_, models::MergeCache>,
) -> Result<models::SkipMergeGroupResult, String> {
	let cache = cache.inner().clone();
	let group_id_for_error = group_id.clone();
	tauri::async_runtime::spawn_blocking(move || {
		services::merge_groups::skip_merge_cached_group(&group_id, &cache)
	})
		.await
		.map_err(|e| {
			format!(
				"Failed to execute skip-merge task for group '{}': {e}",
				group_id_for_error
			)
		})?
		.map_err(|e| format!("Skip-merge failed for group '{}': {e}", group_id_for_error))
}




#[tauri::command]
pub async fn find_groups_between_db_and_files(
    paths: Vec<String>,
    cache: tauri::State<'_, models::MergeCache>,
) -> std::result::Result<(), String> {
    let cache = cache.inner().clone();

    tauri::async_runtime::spawn_blocking(move || -> std::result::Result<(), String> {
        println!("Starting DB/file grouping. paths={:?}", paths);

        // actually CALL the function
        services::db_ingestion::find_groups_between_db_and_files(paths, &cache)?;

        println!("DB/file grouping finished");
        Ok(())
    })
    .await
    .map_err(|e| format!("Failed to execute CSV processing task: {e}"))??;

    Ok(())
}

// ...existing code...