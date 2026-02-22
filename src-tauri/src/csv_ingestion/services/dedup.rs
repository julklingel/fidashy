use crate::csv_ingestion::models;
use crate::csv_ingestion::services::grouping;
use polars::prelude::*;
use std::fmt::Display;

pub struct MergeDedupComputation {
    pub deduplicated_df: DataFrame,
    pub input_rows: usize,
    pub merged_rows: usize,
    pub duplicate_rows_removed: usize,
    pub merged_headers: Vec<String>,
}



fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{context}: {e}"))
}

fn count_duplicates_from_lazy_frame(combined_lf: LazyFrame) -> Result<usize, String> {
    let combined_df = with_context(
        combined_lf.clone().collect(),
        "Failed to materialize combined CSV rows for duplicate analysis",
    )?;
    let deduplicated_df = with_context(
        combined_lf
            .unique(None, UniqueKeepStrategy::First)
            .collect(),
        "Failed to calculate duplicate rows with Polars lazy unique",
    )?;

    Ok(combined_df.height().saturating_sub(deduplicated_df.height()))
}

pub fn build_merged_deduplicated_frame(paths: &[String]) -> Result<MergeDedupComputation, String> {
    if paths.is_empty() {
        return Err("At least one file is required to process CSV data".to_string());
    }

    let combined_lf = grouping::combine_paths_lazy(paths)?;

    let combined_df = with_context(
        combined_lf.clone().collect(),
        "Failed to materialize combined CSV rows for merge",
    )?;
    let deduplicated_df = with_context(
        combined_lf
            .unique(None, UniqueKeepStrategy::First)
            .collect(),
        "Failed to remove duplicate rows with Polars lazy unique",
    )?;

    let input_rows = combined_df.height();
    let merged_rows = deduplicated_df.height();
    let duplicate_rows_removed = input_rows.saturating_sub(merged_rows);
    let merged_headers = deduplicated_df
        .get_column_names_owned()
        .into_iter()
        .map(|name| name.to_string())
        .collect();

    Ok(MergeDedupComputation {
        deduplicated_df,
        input_rows,
        merged_rows,
        duplicate_rows_removed,
        merged_headers,
    })
}

pub fn remove_duplicates_in_group(group: &models::MatchingHeaderGroup) -> Result<usize, String> {
    if group.file_paths.len() < 2 {
        return Ok(0);
    }

    let combined_lf = grouping::combine_group_frames_lazy(group)?;
    count_duplicates_from_lazy_frame(combined_lf)
}

pub fn annotate_group_duplicates(
    groups: Vec<models::MatchingHeaderGroup>,
) -> Result<Vec<models::MatchingHeaderGroup>, String> {
    groups
        .into_iter()
        .map(|mut group| {
            group.duplicate_rows = remove_duplicates_in_group(&group)?;
            Ok(group)
        })
        .collect()
}



