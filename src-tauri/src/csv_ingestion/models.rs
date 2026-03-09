use serde::{Deserialize, Serialize};
use polars::prelude::DataFrame;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupWithDuplicates {
    pub group_id: String,
    pub paths: Vec<String>,
    pub duplicate_count: usize,
    pub total_entries: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeduplicateGroupResult {
    pub group_id: String,
    pub source_file_count: usize,
    pub rows_before: usize,
    pub rows_after: usize,
    pub duplicates_removed: usize,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkipMergeGroupResult {
    pub group_id: String,
    pub source_file_count: usize,
    pub standalone_paths: Vec<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbSchemaCheckResult {
    pub target_table: String,
    pub table_exists: bool,
    pub schema_matches: bool,
    pub source_columns: Vec<String>,
    pub existing_columns: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbImportActionResult {
    pub target_table: String,
    pub rows_written: usize,
    pub source_label: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CachedDataFrame {
    pub paths: Vec<String>,
    pub data_frame: DataFrame,
}

#[derive(Debug, Clone, Default)]
pub struct MergeCache {
    inner: Arc<Mutex<HashMap<String, CachedDataFrame>>>,
}

impl MergeCache {
    pub fn insert(&self, group_id: String, cached_data_frame: CachedDataFrame) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("merge cache lock poisoned: {e}"))?;
        guard.insert(group_id, cached_data_frame);
        Ok(())
    }

    pub fn get_group(&self, group_id: &str) -> Result<Option<CachedDataFrame>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| format!("merge cache lock poisoned: {e}"))?;

        Ok(guard.get(group_id).cloned())
    }

    pub fn remove_group(&self, group_id: &str) -> Result<Option<CachedDataFrame>, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("merge cache lock poisoned: {e}"))?;

        Ok(guard.remove(group_id))
    }
}