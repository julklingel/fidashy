use polars::prelude::{IntoLazy, UniqueKeepStrategy};
use crate::csv_ingestion::models::{
    CachedDataFrame,
    DeduplicateGroupResult,
    MergeCache,
    SkipMergeGroupResult,
};

pub fn deduplicate_cached_group(
    group_id: &str,
    cache: &MergeCache,
) -> Result<DeduplicateGroupResult, String> {
    let cached_group = cache
        .get_group(group_id)?
        .ok_or_else(|| format!("No cached group found for id: {group_id}"))?;

    let rows_before = cached_group.data_frame.height();
    let source_file_count = cached_group.paths.len();


    let deduped_df = cached_group
        .data_frame
        .clone()
        .lazy()
        .unique_stable(None, UniqueKeepStrategy::First)
        .collect()
        .map_err(|e| format!("Failed to deduplicate group '{group_id}': {e}"))?;

    let rows_after = deduped_df.height();
    let duplicates_removed = rows_before.saturating_sub(rows_after);


    cache.insert(
        group_id.to_string(),
        CachedDataFrame {
            paths: cached_group.paths.clone(),
            data_frame: deduped_df.clone(),
        },
    )?;


    println!(
        "Merge ->  Rows before: {}, rows after dedupe: {}",

        rows_before,
        rows_after
    );

    Ok(DeduplicateGroupResult {
        group_id: group_id.to_string(),
        source_file_count,
        rows_before,
        rows_after,
        duplicates_removed,
        message: format!(
            "Removed {duplicates_removed} duplicate rows from group '{group_id}'"
        ),
    })
}

pub fn skip_merge_cached_group(
    group_id: &str,
    cache: &MergeCache,
) -> Result<SkipMergeGroupResult, String> {
    let removed_group = cache
        .remove_group(group_id)?
        .ok_or_else(|| format!("No cached group found for id: {group_id}"))?;

    let source_file_count = removed_group.paths.len();

    Ok(SkipMergeGroupResult {
        group_id: group_id.to_string(),
        source_file_count,
        standalone_paths: removed_group.paths.clone(),
        message: format!(
            "Group '{group_id}' will stay as {source_file_count} standalone table(s)"
        ),
    })
}