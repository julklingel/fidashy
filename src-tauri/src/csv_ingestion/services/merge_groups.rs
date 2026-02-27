use crate::csv_ingestion::models::MergeCache;

pub fn merge_csv(group_id: &str, cache: &MergeCache) -> Result<String, String> {
    let cached_group = cache
        .get_group(group_id)?
        .ok_or_else(|| format!("No cached group found for id: {group_id}"))?;

    let _rows = cached_group.merged_df.height();
    let _source_files = cached_group.paths.len();

    Ok("ok".to_string())
}
