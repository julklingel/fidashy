use crate::csv_ingestion::models;
use crate::csv_ingestion::services::grouping;
use polars::prelude::*;
use std::fmt::Display;

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

pub fn remove_duplicates_in_group(group: &models::MatchingHeaderGroup) -> Result<usize, String> {
    if group.file_paths.len() < 2 {
        return Ok(0);
    }

    let combined_lf = grouping::combine_paths_lazy(&group.file_paths)?;
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
