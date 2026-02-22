use crate::csv_ingestion::models;
use crate::csv_ingestion::services::data_sniff;
use polars::prelude::*;
use std::path::Path;
use std::collections::BTreeMap;
use std::fmt::Display;

fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{context}: {e}"))
}

fn scan_csv_lazy(path: &str) -> Result<LazyFrame, String> {
    let separator = data_sniff::sniff_separator(Path::new(path))?;

    with_context(
        LazyCsvReader::new(PlRefPath::new(path))
            .with_has_header(true)
            .with_separator(separator)
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_ignore_errors(true)
            .finish(),
        "Failed to scan CSV file for group operations",
    )
}

pub(crate) fn combine_paths_lazy(paths: &[String]) -> Result<LazyFrame, String> {
    if paths.is_empty() {
        return Err("No files found in matching group".to_string());
    }

    let lazy_frames: Vec<LazyFrame> = paths
        .iter()
        .map(|path| scan_csv_lazy(path))
        .collect::<Result<Vec<_>, _>>()?;

    with_context(
        concat(lazy_frames, UnionArgs::default()),
        "Failed to combine CSV lazy frames",
    )
}

pub fn group_files_with_matching_headers(
    files: &[models::CsvFileSchemaInfo],
) -> Vec<models::MatchingHeaderGroup> {
    let mut grouped: BTreeMap<u64, models::MatchingHeaderGroup> = BTreeMap::new();

    for file in files {
        let group = grouped
            .entry(file.schema_signature_hash)
            .or_insert_with(|| models::MatchingHeaderGroup {
                headers: file.headers.clone(),
                file_paths: Vec::new(),
                duplicate_rows: 0,
            });

        group.file_paths.push(file.path.clone());
    }

    grouped
        .into_values()
        .filter(|group| group.file_paths.len() > 1)
        .collect()
}
