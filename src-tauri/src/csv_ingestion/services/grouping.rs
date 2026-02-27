use polars::prelude::*;
use std::collections::HashMap;
use crate::csv_ingestion::models::GroupWithDuplicates;

pub fn lazy_grouping_csv_many(
    paths: Vec<String>,
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

    let grouped_paths: Vec<Vec<String>> = groups.into_values().collect();
    let grouped_path_filtered: Vec<Vec<String>> = grouped_paths
        .into_iter()
        .filter(|group| group.len()>1)
        .collect();

    let dup_stats = duplicate_count_per_group(&grouped_path_filtered)?;

    let enriched = grouped_path_filtered
        .into_iter()
        .enumerate()
        .map(|(group_idx, paths)| GroupWithDuplicates {
            paths,
            duplicate_count: dup_stats.get(&group_idx).map(|(duplicates, _)| *duplicates).unwrap_or(0),
            total_entries: dup_stats.get(&group_idx).map(|(_, total)| *total).unwrap_or(0),
        })
        .collect();

    Ok(enriched)
}


pub fn duplicate_count_per_group(
    grouped_path_filtered: &[Vec<String>],
) -> PolarsResult<HashMap<usize, (usize, usize)>> {
    let mut out: HashMap<usize, (usize, usize)> = HashMap::new();

    for (group_idx, paths) in grouped_path_filtered.iter().enumerate() {
        let mut lfs: Vec<LazyFrame> = Vec::new();

        for path in paths {
            let lf = LazyCsvReader::new(PlRefPath::from(path.as_str()))
                .with_has_header(true)
                .finish()?;
            lfs.push(lf);
        }

        if lfs.is_empty() {
            continue;
        }

        let df = concat(lfs, UnionArgs::default())?.collect()?;
        let dup_mask = df.is_duplicated()?;
        let dup_count = dup_mask.sum().unwrap_or(0) as usize;
        let total_entries = df.height();

        out.insert(group_idx, (dup_count, total_entries));
    }

    Ok(out)
}


// pub fn vertical_concatenation( paths: Vec<String>) -> PolarsResult<Vec<String>> {

// }



