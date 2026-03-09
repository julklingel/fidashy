use super::{models, services};

use crate::db::DuckDbState;
use tauri::State;

#[tauri::command]
pub async fn lazy_grouping_csv_many(
    paths: Vec<String>,
    cache: tauri::State<'_, models::MergeCache>,
) -> Result<Vec<models::GroupWithDuplicates>, String> {
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
    .map_err(|e| {
        format!(
            "Deduplication failed for group '{}': {e}",
            group_id_for_error
        )
    })
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
pub async fn create_new_table_from_source(
    source_path: String,
    preferred_table_name: String,
    db_state: State<'_, DuckDbState>,
) -> Result<(), String> {
    let db_state = db_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        services::db_ingestion::create_new_table_from_source(
            source_path,
            preferred_table_name,
            &db_state,
        )
    })
    .await
    .map_err(|e| format!("Failed to execute create-table task: {e}"))?
}

// #[tauri::command]
// pub async fn merge_source_into_table(
//     source_kind: String,
//     source_name: String,
//     source_paths: Vec<String>,
//     target_table: String,
//     cache: tauri::State<'_, models::MergeCache>,
//     db_state: State<'_, DuckDbState>,
// ) -> Result<models::DbImportActionResult, String> {
//     let cache = cache.inner().clone();
//     let db_state = db_state.inner().clone();

//     tauri::async_runtime::spawn_blocking(move || {
//         let source_kind = services::db_ingestion::ImportSourceKind::try_from(source_kind.as_str())?;
//         services::db_ingestion::merge_source_into_table(
//             source_kind,
//             source_name,
//             source_paths,
//             target_table,
//             &cache,
//             &db_state,
//         )
//     })
//     .await
//     .map_err(|e| format!("Failed to execute merge task: {e}"))?
// }

#[tauri::command]
pub async fn find_groups_between_db_and_files(
    paths: Vec<String>,
    cache_ids: Vec<String>,
    cache: tauri::State<'_, models::MergeCache>,
    db_state: State<'_, DuckDbState>,
) -> Result<(), String> {
    let cache = cache.inner().clone();
    let db_state = db_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        services::db_ingestion::find_groups_between_db_and_files(
            paths, cache_ids, &cache, &db_state,
        )
    })
    .await
    .map_err(|e| format!("Failed to execute DB/file grouping task: {e}"))??;

    Ok(())
}
