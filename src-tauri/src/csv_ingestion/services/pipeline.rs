use crate::csv_ingestion::models;
use crate::csv_ingestion::services::dedup;
use crate::csv_ingestion::services::grouping;
use crate::csv_ingestion::services::schema;
use crate::db::models::DuckDbState;
use duckdb::params_from_iter;
use duckdb::types::ValueRef;
use polars::prelude::AnyValue;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetColumnType {
    Boolean,
    Int64,
    Float64,
    Date,
    Datetime,
    Text,
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn normalize_headers(headers: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = headers
        .iter()
        .map(|header| header.trim().to_ascii_lowercase())
        .collect();
    normalized.sort_unstable();
    normalized
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }

    let collapsed = out
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if collapsed.is_empty() {
        "csv_group".to_string()
    } else {
        collapsed
    }
}

fn auto_table_name(merged_headers: &[String]) -> String {
    let header_prefix = merged_headers
        .first()
        .map(|header| sanitize_identifier(header))
        .unwrap_or_else(|| "csv_group".to_string());
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    format!("{}_{}", header_prefix, ts)
}

fn values_to_key(values: &[Option<String>]) -> String {
    values
        .iter()
        .map(|value| value.clone().unwrap_or_else(|| "<NULL>".to_string()))
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn strip_wrapping_quotes(mut value: String) -> String {
    loop {
        if value.len() < 2 {
            return value;
        }

        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            value = value[1..value.len() - 1].to_string();
            continue;
        }

        return value;
    }
}

fn normalize_raw_string(raw: &str) -> Option<String> {
    let mut normalized = raw.trim().to_string();
    if normalized.is_empty() {
        return None;
    }

    normalized = strip_wrapping_quotes(normalized);
    normalized = normalized.replace("\"\"", "\"").trim().to_string();
    if normalized.is_empty() {
        return None;
    }

    let lower = normalized.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "none" | "n/a" | "na" | "nan" | "-"
    ) {
        return None;
    }

    Some(normalized)
}

fn normalize_numeric_candidate(value: &str) -> Option<String> {
    let compact = value.replace([' ', '_'], "");
    let has_dot = compact.contains('.');
    let has_comma = compact.contains(',');

    if has_dot && has_comma {
        let last_dot = compact.rfind('.').unwrap_or(0);
        let last_comma = compact.rfind(',').unwrap_or(0);
        if last_dot > last_comma {
            Some(compact.replace(',', ""))
        } else {
            Some(compact.replace('.', "").replace(',', "."))
        }
    } else if has_comma {
        let comma_count = compact.matches(',').count();
        if comma_count > 1 {
            Some(compact.replace(',', ""))
        } else {
            let split: Vec<&str> = compact.split(',').collect();
            if split.len() == 2 && split[1].len() == 3 {
                Some(compact.replace(',', ""))
            } else {
                Some(compact.replace(',', "."))
            }
        }
    } else {
        Some(compact)
    }
}

fn normalize_datetime_candidate(value: &str) -> String {
    let mut normalized = value.trim().replace('T', " ");
    normalized = normalized.trim_end_matches('Z').to_string();
    strip_wrapping_quotes(normalized)
}

fn normalize_date_candidate(value: &str) -> String {
    let normalized = normalize_datetime_candidate(value);
    normalized
        .split(' ')
        .next()
        .unwrap_or(normalized.as_str())
        .to_string()
}

fn value_requires_float(value: &AnyValue<'_>) -> bool {
    let Some(normalized) = normalize_raw_string(&value.to_string()) else {
        return false;
    };
    let Some(candidate) = normalize_numeric_candidate(&normalized) else {
        return false;
    };

    if candidate.parse::<i64>().is_ok() {
        return false;
    }

    if let Ok(parsed_float) = candidate.parse::<f64>() {
        return parsed_float.fract().abs() >= f64::EPSILON;
    }

    false
}

fn detect_int_columns_requiring_float_widening(
    merged_df: &polars::prelude::DataFrame,
    column_types: &[TargetColumnType],
) -> Vec<usize> {
    let mut indices_to_widen = Vec::new();

    for (index, column_type) in column_types.iter().enumerate() {
        if *column_type != TargetColumnType::Int64 {
            continue;
        }

        let Some(column) = merged_df.columns().get(index) else {
            continue;
        };

        let series = column.as_materialized_series();
        if series.iter().any(|value| value_requires_float(&value)) {
            indices_to_widen.push(index);
        }
    }

    indices_to_widen
}

fn widen_table_columns_to_float(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    indices_to_widen: &[usize],
) -> Result<(), String> {
    if indices_to_widen.is_empty() {
        return Ok(());
    }

    db_state.with_db(|db| {
        for index in indices_to_widen {
            let column_name = merged_headers
                .get(*index)
                .ok_or_else(|| format!("Failed to resolve column index {index} for widening"))?;
            let sql = format!(
                "ALTER TABLE {} ALTER COLUMN {} SET DATA TYPE DOUBLE",
                quote_identifier(table_name),
                quote_identifier(column_name)
            );

            db.execute(&sql, []).map_err(|e| {
                format!(
                    "Failed to widen column '{}' to DOUBLE in table '{}': {e}",
                    column_name, table_name
                )
            })?;
        }

        Ok(())
    })
}

fn target_type_from_inferred(inferred: &schema::InferredType) -> TargetColumnType {
    match inferred {
        schema::InferredType::Boolean => TargetColumnType::Boolean,
        schema::InferredType::Int64 => TargetColumnType::Int64,
        schema::InferredType::Float64 => TargetColumnType::Float64,
        schema::InferredType::Date => TargetColumnType::Date,
        schema::InferredType::Datetime => TargetColumnType::Datetime,
        schema::InferredType::Utf8 => TargetColumnType::Text,
    }
}

fn target_type_from_duckdb_type(data_type: &str) -> TargetColumnType {
    let upper = data_type.to_ascii_uppercase();
    if upper.contains("BOOL") {
        TargetColumnType::Boolean
    } else if upper.contains("DOUBLE") || upper.contains("FLOAT") || upper.contains("DECIMAL") {
        TargetColumnType::Float64
    } else if upper.contains("INT") {
        TargetColumnType::Int64
    } else if upper.contains("TIMESTAMP") || upper.contains("DATETIME") {
        TargetColumnType::Datetime
    } else if upper == "DATE" {
        TargetColumnType::Date
    } else {
        TargetColumnType::Text
    }
}

fn any_value_to_db_value_for_type(
    value: &AnyValue<'_>,
    target_type: TargetColumnType,
    column_name: &str,
) -> Result<Option<String>, String> {
    coerce_raw_value_for_type(Some(value.to_string()), target_type, column_name)
}

fn coerce_raw_value_for_type(
    raw_value: Option<String>,
    target_type: TargetColumnType,
    column_name: &str,
) -> Result<Option<String>, String> {
    let Some(raw) = raw_value else {
        return Ok(None);
    };

    let Some(normalized) = normalize_raw_string(&raw) else {
        return Ok(None);
    };

    let coerced = match target_type {
        TargetColumnType::Text => normalized,
        TargetColumnType::Boolean => {
            let lower = normalized.to_ascii_lowercase();
            if matches!(lower.as_str(), "true" | "t" | "yes" | "y" | "1") {
                "true".to_string()
            } else if matches!(lower.as_str(), "false" | "f" | "no" | "n" | "0") {
                "false".to_string()
            } else {
                return Err(format!("Could not convert value '{normalized}' to BOOLEAN in column '{column_name}'"));
            }
        }
        TargetColumnType::Int64 => {
            let Some(candidate) = normalize_numeric_candidate(&normalized) else {
                return Err(format!("Could not convert value '{normalized}' to INT64 in column '{column_name}'"));
            };

            if let Ok(parsed) = candidate.parse::<i64>() {
                parsed.to_string()
            } else if let Ok(parsed_float) = candidate.parse::<f64>() {
                if parsed_float.fract().abs() < f64::EPSILON {
                    (parsed_float as i64).to_string()
                } else {
                    return Err(format!("Could not convert value '{normalized}' to INT64 in column '{column_name}'"));
                }
            } else {
                return Err(format!("Could not convert value '{normalized}' to INT64 in column '{column_name}'"));
            }
        }
        TargetColumnType::Float64 => {
            let Some(candidate) = normalize_numeric_candidate(&normalized) else {
                return Err(format!("Could not convert value '{normalized}' to DOUBLE in column '{column_name}'"));
            };
            let parsed = candidate
                .parse::<f64>()
                .map_err(|_| format!("Could not convert value '{normalized}' to DOUBLE in column '{column_name}'"))?;
            parsed.to_string()
        }
        TargetColumnType::Date => normalize_date_candidate(&normalized),
        TargetColumnType::Datetime => {
            let candidate = normalize_datetime_candidate(&normalized);
            if candidate.contains(' ') {
                candidate
            } else {
                format!("{candidate} 00:00:00")
            }
        }
    };

    Ok(Some(coerced))
}

fn value_ref_to_optional_string(value: ValueRef<'_>) -> Option<String> {
    match value {
        ValueRef::Null => None,
        ValueRef::Text(v) => Some(String::from_utf8_lossy(v).to_string()),
        _ => Some(value_ref_to_key_part(value)),
    }
}

fn ensure_table_does_not_exist(db_state: &DuckDbState, table_name: &str) -> Result<(), String> {
    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(
                "SELECT 1 FROM information_schema.tables WHERE table_schema = 'main' AND table_name = ? LIMIT 1",
            )
            .map_err(|e| format!("Failed to prepare table existence query: {e}"))?;

        let mut rows = stmt
            .query([table_name])
            .map_err(|e| format!("Failed to query table existence: {e}"))?;

        let exists = rows
            .next()
            .map_err(|e| format!("Failed to iterate table existence rows: {e}"))?
            .is_some();

        if exists {
            Err(format!("Table '{table_name}' already exists"))
        } else {
            Ok(())
        }
    })
}

fn ensure_table_headers_match(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
) -> Result<(), String> {
    let target = normalize_headers(merged_headers);

    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(
                "SELECT column_name FROM information_schema.columns WHERE table_schema = 'main' AND table_name = ? ORDER BY ordinal_position",
            )
            .map_err(|e| format!("Failed to prepare table column query: {e}"))?;

        let columns: Vec<String> = stmt
            .query_map([table_name], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query table columns: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect table columns: {e}"))?;

        if columns.is_empty() {
            return Err(format!("Table '{table_name}' does not exist or has no columns"));
        }

        if normalize_headers(&columns) != target {
            return Err(format!(
                "Table '{table_name}' columns do not match merged CSV headers"
            ));
        }

        Ok(())
    })
}

fn create_table_with_inferred_columns(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    inferred_columns: &[schema::InferredColumn],
) -> Result<(), String> {
    let inferred_by_name: HashMap<&str, &schema::InferredType> = inferred_columns
        .iter()
        .map(|column| (column.name.as_str(), &column.inferred))
        .collect();

    let column_definitions = merged_headers
        .iter()
        .map(|header| {
            let sql_type = inferred_by_name
                .get(header.as_str())
                .map(|inferred| inferred.to_duckdb_type())
                .unwrap_or("TEXT");

            format!("{} {sql_type}", quote_identifier(header))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "CREATE TABLE {} ({column_definitions})",
        quote_identifier(table_name)
    );

    db_state.with_db(|db| {
        db.execute(&sql, [])
            .map_err(|e| format!("Failed to create table '{table_name}': {e}"))?;
        Ok(())
    })
}

fn load_existing_row_keys(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    column_types: &[TargetColumnType],
) -> Result<HashSet<String>, String> {
    let selected_columns = merged_headers
        .iter()
        .map(|column| format!("CAST({} AS VARCHAR)", quote_identifier(column)))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT {selected_columns} FROM {}", quote_identifier(table_name));

    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare existing key query: {e}"))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| format!("Failed to query existing table rows: {e}"))?;

        let mut keys = HashSet::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| format!("Failed to iterate existing table rows: {e}"))?
        {
            let mut parts = Vec::with_capacity(merged_headers.len());
            for idx in 0..merged_headers.len() {
                let value = row
                    .get_ref(idx)
                    .map_err(|e| format!("Failed to read existing table row value: {e}"))?;
                let column_name = merged_headers
                    .get(idx)
                    .map_or("<unknown>", String::as_str);
                let target_type = column_types
                    .get(idx)
                    .copied()
                    .unwrap_or(TargetColumnType::Text);
                let coerced = coerce_raw_value_for_type(
                    value_ref_to_optional_string(value),
                    target_type,
                    column_name,
                )?;
                parts.push(coerced.unwrap_or_else(|| "<NULL>".to_string()));
            }

            keys.insert(parts.join("\u{1f}"));
        }

        Ok(keys)
    })
}

fn load_table_column_types(
    db_state: &DuckDbState,
    table_name: &str,
) -> Result<HashMap<String, TargetColumnType>, String> {
    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(
                "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = 'main' AND table_name = ? ORDER BY ordinal_position",
            )
            .map_err(|e| format!("Failed to prepare table type query: {e}"))?;

        let rows = stmt
            .query_map([table_name], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to query table column types: {e}"))?;

        let mut result = HashMap::new();
        for row in rows {
            let (column_name, data_type) =
                row.map_err(|e| format!("Failed to read table column type row: {e}"))?;
            result.insert(column_name, target_type_from_duckdb_type(&data_type));
        }

        Ok(result)
    })
}

fn insert_non_duplicate_rows(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    merged_df: &polars::prelude::DataFrame,
    existing_keys: &mut HashSet<String>,
    column_types: &[TargetColumnType],
) -> Result<(usize, usize), String> {
    let quoted_columns = merged_headers
        .iter()
        .map(|header| quote_identifier(header))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = (0..merged_headers.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let insert_sql = format!(
        "INSERT INTO {} ({quoted_columns}) VALUES ({placeholders})",
        quote_identifier(table_name)
    );

    let mut rows_inserted = 0usize;
    let mut rows_skipped_duplicates = 0usize;

    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(&insert_sql)
            .map_err(|e| format!("Failed to prepare insert statement: {e}"))?;

        for row_index in 0..merged_df.height() {
            let row = merged_df
                .get_row(row_index)
                .map_err(|e| format!("Failed to read merged DataFrame row: {e}"))?;
            let mut converted_values = Vec::with_capacity(row.0.len());
            for (idx, value) in row.0.iter().enumerate() {
                let target_type = column_types
                    .get(idx)
                    .copied()
                    .unwrap_or(TargetColumnType::Text);
                let column_name = merged_headers
                    .get(idx)
                    .map_or("<unknown>", String::as_str);

                let coerced = any_value_to_db_value_for_type(value, target_type, column_name)
                    .map_err(|e| format!("Failed to convert row for '{table_name}': {e}"))?;
                converted_values.push(coerced);
            }

            let row_key = values_to_key(&converted_values);

            if existing_keys.contains(&row_key) {
                rows_skipped_duplicates += 1;
                continue;
            }

            stmt.execute(params_from_iter(converted_values.iter()))
                .map_err(|e| format!("Failed to insert row into '{table_name}': {e}"))?;

            existing_keys.insert(row_key);
            rows_inserted += 1;
        }

        Ok(())
    })?;

    Ok((rows_inserted, rows_skipped_duplicates))
}

fn find_matching_table_by_headers(
    db_state: &DuckDbState,
    merged_headers: &[String],
) -> Result<Option<String>, String> {
    let target = normalize_headers(merged_headers);

    db_state.with_db(|db| {
        let mut table_stmt = db
            .prepare(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_type = 'BASE TABLE' ORDER BY table_name",
            )
            .map_err(|e| format!("Failed to query DuckDB table list: {e}"))?;

        let table_names: Vec<String> = table_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to iterate DuckDB table list: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect DuckDB table list: {e}"))?;

        for table_name in table_names {
            let mut col_stmt = db
                .prepare(
                    "SELECT column_name FROM information_schema.columns WHERE table_schema = 'main' AND table_name = ? ORDER BY ordinal_position",
                )
                .map_err(|e| format!("Failed to prepare table column query: {e}"))?;

            let columns: Vec<String> = col_stmt
                .query_map([table_name.as_str()], |row| row.get::<_, String>(0))
                .map_err(|e| format!("Failed to query table columns: {e}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect table columns: {e}"))?;

            if normalize_headers(&columns) == target {
                return Ok(Some(table_name));
            }
        }

        Ok(None)
    })
}

fn value_ref_to_key_part(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => "<NULL>".to_string(),
        ValueRef::Boolean(v) => v.to_string(),
        ValueRef::TinyInt(v) => v.to_string(),
        ValueRef::SmallInt(v) => v.to_string(),
        ValueRef::Int(v) => v.to_string(),
        ValueRef::BigInt(v) => v.to_string(),
        ValueRef::HugeInt(v) => v.to_string(),
        ValueRef::UTinyInt(v) => v.to_string(),
        ValueRef::USmallInt(v) => v.to_string(),
        ValueRef::UInt(v) => v.to_string(),
        ValueRef::UBigInt(v) => v.to_string(),
        ValueRef::Float(v) => v.to_string(),
        ValueRef::Double(v) => v.to_string(),
        ValueRef::Decimal(v) => v.to_string(),
        ValueRef::Timestamp(_, v) => v.to_string(),
        ValueRef::Text(v) => String::from_utf8_lossy(v).to_string(),
        ValueRef::Blob(v) => format!("<BLOB:{}>", v.len()),
        ValueRef::Date32(v) => v.to_string(),
        ValueRef::Time64(_, v) => v.to_string(),
        ValueRef::Interval { months, days, nanos } => format!("{months}:{days}:{nanos}"),
        _ => "<UNSUPPORTED>".to_string(),
    }
}

fn count_duplicates_with_table(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    merged_df: &polars::prelude::DataFrame,
) -> Result<usize, String> {
    let existing_column_types = load_table_column_types(db_state, table_name)?;
    let column_types = merged_headers
        .iter()
        .map(|header| {
            existing_column_types
                .get(header)
                .copied()
                .unwrap_or(TargetColumnType::Text)
        })
        .collect::<Vec<_>>();

    let mut merged_keys = HashSet::with_capacity(merged_df.height());
    for row_index in 0..merged_df.height() {
        let row = merged_df
            .get_row(row_index)
            .map_err(|e| format!("Failed to read merged DataFrame row: {e}"))?;

        let mut converted_values = Vec::with_capacity(row.0.len());
        for (idx, value) in row.0.iter().enumerate() {
            let target_type = column_types
                .get(idx)
                .copied()
                .unwrap_or(TargetColumnType::Text);
            let column_name = merged_headers
                .get(idx)
                .map_or("<unknown>", String::as_str);
            let coerced = any_value_to_db_value_for_type(value, target_type, column_name)
                .map_err(|e| format!("Failed to convert merged row for duplicate check in '{table_name}': {e}"))?;
            converted_values.push(coerced);
        }

        merged_keys.insert(values_to_key(&converted_values));
    }

    if merged_keys.is_empty() {
        return Ok(0);
    }

    let selected_columns = merged_headers
        .iter()
        .map(|column| format!("CAST({} AS VARCHAR)", quote_identifier(column)))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT {selected_columns} FROM {}", quote_identifier(table_name));

    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare table duplicate query: {e}"))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| format!("Failed to query table rows for duplicate comparison: {e}"))?;

        let mut duplicates = 0usize;

        while let Some(row) = rows
            .next()
            .map_err(|e| format!("Failed to iterate table rows for duplicate comparison: {e}"))?
        {
            let mut parts = Vec::with_capacity(merged_headers.len());
            for idx in 0..merged_headers.len() {
                let value = row
                    .get_ref(idx)
                    .map_err(|e| format!("Failed to read table row value: {e}"))?;
                let column_name = merged_headers
                    .get(idx)
                    .map_or("<unknown>", String::as_str);
                let target_type = column_types
                    .get(idx)
                    .copied()
                    .unwrap_or(TargetColumnType::Text);
                let coerced = coerce_raw_value_for_type(
                    value_ref_to_optional_string(value),
                    target_type,
                    column_name,
                )
                .map_err(|e| {
                    format!(
                        "Failed to normalize table row for duplicate comparison in '{table_name}': {e}"
                    )
                })?;
                parts.push(coerced.unwrap_or_else(|| "<NULL>".to_string()));
            }

            let key = parts.join("\u{1f}");
            if merged_keys.contains(&key) {
                duplicates += 1;
            }
        }

        Ok(duplicates)
    })
}



pub fn process_csv_files(paths: Vec<String>) -> Result<models::ProcessCsvResult, String> {
    if paths.is_empty() {
        return Err("No CSV file paths provided".to_string());
    }

    let files_with_schema: Vec<models::CsvFileSchemaInfo> = paths
        .into_par_iter()
        .map(schema::collect_file_schema)
        .collect::<Result<Vec<_>, _>>()?;


    let processed_files = files_with_schema.len();
    let files = files_with_schema
        .iter()
        .map(|file| models::CsvFileHeaders {
            path: file.path.clone(),
            headers: file.headers.clone(),
        })
        .collect();
    let grouped = grouping::group_files_with_matching_headers(&files_with_schema);
    let matching_header_groups = dedup::annotate_group_duplicates(grouped)?;

    let group_count = matching_header_groups.len();
    let total_duplicate_rows = matching_header_groups
        .iter()
        .map(|group| group.duplicate_rows)
        .sum();

    Ok(models::ProcessCsvResult {
        processed_files,
        group_count,
        total_duplicate_rows,
        files,
        matching_header_groups,
    })
}


pub fn merge_csv_group(paths: Vec<String>, db_state: &DuckDbState) -> Result<models::MergeCsvGroupResult, String> {
    let merge = dedup::build_merged_deduplicated_frame(&paths)?;

    let matching_table_name = find_matching_table_by_headers(db_state, &merge.merged_headers)?;
    let duplicate_rows_with_db = if let Some(table_name) = matching_table_name.as_deref() {
        count_duplicates_with_table(
            db_state,
            table_name,
            &merge.merged_headers,
            &merge.deduplicated_df,
        )?
    } else {
        0
    };

    Ok(models::MergeCsvGroupResult {
        input_rows: merge.input_rows,
        merged_rows: merge.merged_rows,
        duplicate_rows_removed: merge.duplicate_rows_removed,
        merged_columns: merge.merged_headers.len(),
        merged_headers: merge.merged_headers,
        matching_table_name: matching_table_name.clone(),
        duplicate_rows_with_db,
        requires_user_choice: matching_table_name.is_some(),
    })
}

pub fn create_table_from_csv_group(
    paths: Vec<String>,
    suggested_table_name: Option<String>,
    db_state: &DuckDbState,
) -> Result<models::CsvIngestionWriteResult, String> {
    let merge = dedup::build_merged_deduplicated_frame(&paths)?;
    let inferred_columns = schema::infer_dataframe_schema(&merge.deduplicated_df, 5000);
    let inferred_type_by_name: HashMap<&str, TargetColumnType> = inferred_columns
        .iter()
        .map(|column| (column.name.as_str(), target_type_from_inferred(&column.inferred)))
        .collect();
    let column_types = merge
        .merged_headers
        .iter()
        .map(|header| {
            inferred_type_by_name
                .get(header.as_str())
                .copied()
                .unwrap_or(TargetColumnType::Text)
        })
        .collect::<Vec<_>>();

    let table_name = suggested_table_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_identifier)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| auto_table_name(&merge.merged_headers));

    ensure_table_does_not_exist(db_state, &table_name)?;
    create_table_with_inferred_columns(
        db_state,
        &table_name,
        &merge.merged_headers,
        &inferred_columns,
    )?;

    let mut existing_keys = HashSet::new();
    let (rows_inserted, rows_skipped_duplicates) = insert_non_duplicate_rows(
        db_state,
        &table_name,
        &merge.merged_headers,
        &merge.deduplicated_df,
        &mut existing_keys,
        &column_types,
    )?;

    Ok(models::CsvIngestionWriteResult {
        table_name,
        input_rows: merge.input_rows,
        rows_inserted,
        rows_skipped_duplicates,
        created_new_table: true,
    })
}

pub fn merge_csv_group_into_existing_table(
    paths: Vec<String>,
    table_name: String,
    db_state: &DuckDbState,
) -> Result<models::CsvIngestionWriteResult, String> {
    if table_name.trim().is_empty() {
        return Err("A target table name is required for merge".to_string());
    }

    let merge = dedup::build_merged_deduplicated_frame(&paths)?;
    ensure_table_headers_match(db_state, &table_name, &merge.merged_headers)?;
    let existing_column_types = load_table_column_types(db_state, &table_name)?;
    let mut column_types = merge
        .merged_headers
        .iter()
        .map(|header| {
            existing_column_types
                .get(header)
                .copied()
                .unwrap_or(TargetColumnType::Text)
        })
        .collect::<Vec<_>>();

    let indices_to_widen = detect_int_columns_requiring_float_widening(
        &merge.deduplicated_df,
        &column_types,
    );
    widen_table_columns_to_float(
        db_state,
        &table_name,
        &merge.merged_headers,
        &indices_to_widen,
    )?;
    for index in indices_to_widen {
        if let Some(column_type) = column_types.get_mut(index) {
            *column_type = TargetColumnType::Float64;
        }
    }

    let mut existing_keys = load_existing_row_keys(
        db_state,
        &table_name,
        &merge.merged_headers,
        &column_types,
    )?;
    let (rows_inserted, rows_skipped_duplicates) = insert_non_duplicate_rows(
        db_state,
        &table_name,
        &merge.merged_headers,
        &merge.deduplicated_df,
        &mut existing_keys,
        &column_types,
    )?;

    Ok(models::CsvIngestionWriteResult {
        table_name,
        input_rows: merge.input_rows,
        rows_inserted,
        rows_skipped_duplicates,
        created_new_table: false,
    })
}