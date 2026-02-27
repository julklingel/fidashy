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

#[derive(Debug)]
pub struct CachedGroup {
    pub merged_df: DataFrame,
    pub paths: Vec<String>,
}

#[derive(Clone, Default)]
pub struct MergeCache {
    inner: Arc<Mutex<HashMap<String, CachedGroup>>>,
}

impl MergeCache {
    pub fn insert(&self, group_id: String, cached_group: CachedGroup) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| format!("merge cache lock poisoned: {e}"))?;
        guard.insert(group_id, cached_group);
        Ok(())
    }

    pub fn get_group(&self, group_id: &str) -> Result<Option<CachedGroup>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| format!("merge cache lock poisoned: {e}"))?;
        Ok(guard.get(group_id).map(|cached| CachedGroup {
            merged_df: cached.merged_df.clone(),
            paths: cached.paths.clone(),
        }))
    }
}
