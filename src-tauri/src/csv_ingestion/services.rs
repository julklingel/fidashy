use super::models;
use polars::prelude::*;
use std::path::{Path, PathBuf};

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

fn collect_headers_with_polars(source_path: &Path) -> Result<Vec<String>, String> {
	let data_frame = CsvReadOptions::default()
		.try_into_reader_with_file_path(Some(source_path.to_path_buf()))
		.map_err(|e| format!("Failed to open CSV with Polars: {e}"))?
		.finish()
		.map_err(|e| format!("Failed to parse CSV with Polars: {e}"))?;

	let headers = data_frame
		.get_column_names_owned()
		.into_iter()
		.map(|name| name.to_string())
		.collect();

	Ok(headers)
}

pub fn process_csv_files(paths: Vec<String>) -> Result<models::ProcessCsvResult, String> {
	if paths.is_empty() {
		return Err("No CSV file paths provided".to_string());
	}

	let mut processed_files = 0usize;
	let mut files: Vec<models::CsvFileHeaders> = Vec::with_capacity(paths.len());

	for path in paths {
		let source_path = PathBuf::from(&path);
		ensure_csv_extension(&source_path)?;

		if !source_path.exists() {
			return Err(format!("CSV file not found: {}", source_path.display()));
		}

		let headers = collect_headers_with_polars(&source_path)?;
		files.push(models::CsvFileHeaders { path, headers });
		processed_files += 1;
	}

	Ok(models::ProcessCsvResult {
		processed_files,
		files,
	})
}
