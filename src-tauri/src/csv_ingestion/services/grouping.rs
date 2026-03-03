use crate::csv_ingestion::models::{CachedDataFrame, GroupWithDuplicates, MergeCache};
use polars::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn workflow_prefix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("wf-{millis}")
}

pub fn lazy_grouping_csv_many(
    paths: Vec<String>,
    cache: &MergeCache,
) -> PolarsResult<Vec<GroupWithDuplicates>> {
    let mut groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

    for path in paths {
        let mut lf = LazyCsvReader::new(PlRefPath::from(path.as_str()))
            .with_has_header(true)
            .finish()?;

        let mut columns = lf
            .collect_schema()?
            .iter_names()
            .map(|s| s.to_string().to_lowercase())
            .collect::<Vec<_>>();

        columns.sort();
        groups.entry(columns).or_default().push(path);
    }

    let grouped_path_filtered: Vec<Vec<String>> = groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();

    let workflow_id = workflow_prefix();

    grouped_path_filtered
        .into_iter()
        .enumerate()
        .map(|(group_idx, group_paths)| {
            let mut lfs: Vec<LazyFrame> = Vec::new();

            for path in &group_paths {
                let lf = LazyCsvReader::new(PlRefPath::from(path.as_str()))
                    .with_has_header(true)
                    .finish()?;
                lfs.push(lf);
            }

            if lfs.is_empty() {
                return Ok(GroupWithDuplicates {
                    group_id: format!("{workflow_id}-{group_idx}"),
                    paths: group_paths,
                    duplicate_count: 0,
                    total_entries: 0,
                });
            }

            let data_frame = concat(lfs, UnionArgs::default())?.collect()?;
            let dup_mask = data_frame.is_duplicated()?;
            let duplicate_count = dup_mask.sum().unwrap_or(0) as usize;
            let total_entries = data_frame.height();
            let group_id = format!("{workflow_id}-{group_idx}");

            cache
                .insert(
                    group_id.clone(),
                    CachedDataFrame {
                        paths: group_paths.clone(),
                        data_frame,
               
                    },
                )
                .map_err(|message| PolarsError::ComputeError(message.into()))?;

            Ok(GroupWithDuplicates {
                group_id,
                paths: group_paths,
                duplicate_count,
                total_entries,
            })
        })
        .collect::<PolarsResult<Vec<GroupWithDuplicates>>>()
}
