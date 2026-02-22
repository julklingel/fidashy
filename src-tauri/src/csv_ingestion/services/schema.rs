use crate::csv_ingestion::models;
use crate::csv_ingestion::services::data_sniff;
pub use crate::csv_ingestion::services::data_sniff::{
    infer_csv_schema,
    infer_csv_schema_with_sniffed_separator,
    infer_dataframe_schema,
    InferredColumn,
    InferredType,
};
use polars::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{context}: {e}"))
}

fn ensure_csv_extension(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| format!("Invalid file extension for {}", path.display()))?;

    if ext != "csv" {
        return Err(format!("Only CSV files are supported: {}", path.display()));
    }

    Ok(())
}

fn collect_headers_with_polars(source_path: &Path) -> Result<(Vec<String>, u64), String> {
    let source_path_str = source_path.to_string_lossy();
    let separator = data_sniff::sniff_separator(source_path)?;
    let mut lazy_frame = with_context(
        LazyCsvReader::new(PlRefPath::new(source_path_str.as_ref()))
            .with_has_header(true)
            .with_separator(separator)
            .with_n_rows(Some(0))
            .finish(),
        "Failed to build lazy CSV reader with Polars",
    )?;

    let schema = with_context(
        lazy_frame.collect_schema(),
        "Failed to infer CSV schema with Polars",
    )?;

    let headers = schema.iter().map(|(name, _)| name.to_string()).collect();

    let mut normalized_fields: Vec<(String, String)> = schema
        .iter()
        .map(|(name, dtype)| {
            (
                name.as_str().trim().to_ascii_lowercase(),
                dtype.to_string(),
            )
        })
        .collect();

    normalized_fields.sort_unstable();
    normalized_fields.dedup();

    let mut hasher = DefaultHasher::new();
    for field in &normalized_fields {
        field.hash(&mut hasher);
    }

    Ok((headers, hasher.finish()))
}

pub fn collect_file_schema(path: String) -> Result<models::CsvFileSchemaInfo, String> {
    let source_path = PathBuf::from(&path);
    ensure_csv_extension(&source_path)?;

    if !source_path.exists() {
        return Err(format!("CSV file not found: {}", source_path.display()));
    }

    let (headers, schema_signature_hash) = collect_headers_with_polars(&source_path)?;

    Ok(models::CsvFileSchemaInfo {
        path,
        headers,
        schema_signature_hash,
    })
}
