use crate::csv_ingestion::models;
use std::collections::BTreeMap;

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
            });

        group.file_paths.push(file.path.clone());
    }

    grouped
        .into_values()
        .filter(|group| group.file_paths.len() > 1)
        .collect()
}
