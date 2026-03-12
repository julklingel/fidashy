
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupProposal {
    pub group_id: String,
    pub paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbMatchProposal {
    pub source_kind: String,
    pub source_id: String,
    pub source_paths: Vec<String>,
    pub columns: Vec<String>,
    pub matching_tables: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTableFromSourceResult {
    pub source_kind: String,
    pub source_id: String,
    pub target_table: String,
    pub rows_before: usize,
    pub rows_after: usize,
    pub duplicates_removed: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MergeSourceIntoTableResult {
    pub source_kind: String,
    pub source_id: String,
    pub target_table: String,
    pub rows_before: usize,
    pub rows_after: usize,
    pub rows_inserted: usize,
    pub duplicates_removed: usize,
}



