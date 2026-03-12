use super::{models, services};
use crate::db::DuckDbState;
use tauri::State;


#[tauri::command]
pub async fn lazy_grouping_csv_many(
    paths: Vec<String>,

) -> Result<Vec<models::GroupProposal>, String> {
    services::lazy_grouping_csv_many(paths)
}




#[tauri::command]
pub async fn create_new_table_from_source(
    source_kind: String,
    source_id: String,
    source_paths: Vec<String>,
    preferred_table_name: String,
    db_state: State<'_, DuckDbState>,
) -> Result<models::CreateTableFromSourceResult, String> {
    let db_state = db_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        services::db_ingestion::create_new_table_from_source(
            source_kind,
            source_id,
            source_paths,
            preferred_table_name,
            &db_state,
        )
    })
    .await
    .map_err(|e| format!("Failed to execute create-table task: {e}"))?
}



#[tauri::command]
pub async fn find_groups_between_db_and_files(
    groups: Vec<models::GroupProposal>,
    standalone_paths: Vec<String>,
    db_state: State<'_, DuckDbState>,
) -> Result<Vec<models::DbMatchProposal>, String> {
    let db_state = db_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<models::DbMatchProposal>, String> {
        services::grouping::find_groups_between_db_and_files(
            groups,
            standalone_paths,
            &db_state,
        )
    })
    .await
    .map_err(|e| format!("Failed to execute DB/file grouping task: {e}"))?
}

#[tauri::command]
pub async fn merge_source_into_table(
    source_kind: String,
    source_id: String,
    source_paths: Vec<String>,
    target_table: String,
    db_state: State<'_, DuckDbState>,
) -> Result<models::MergeSourceIntoTableResult, String> {
    let db_state = db_state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        services::db_ingestion::merge_source_into_table(
            source_kind,
            source_id,
            source_paths,
            target_table,
            &db_state,
        )
    })
    .await
    .map_err(|e| format!("Failed to execute merge-into-table task: {e}"))?
}
