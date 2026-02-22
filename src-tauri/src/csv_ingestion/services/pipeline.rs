use crate::csv_ingestion::models;
use crate::csv_ingestion::services::dedup;
use crate::csv_ingestion::services::grouping;
use crate::csv_ingestion::services::schema;
use crate::db::models::DuckDbState;
use duckdb::params_from_iter;
use duckdb::types::ValueRef;
use polars::prelude::AnyValue;
use rayon::prelude::*;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn dataframe_row_to_key(row: &[AnyValue<'_>]) -> String {
    row.iter()
        .map(any_value_to_key_part)
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn any_value_to_key_part(value: &AnyValue<'_>) -> String {
    match value {
        AnyValue::Null => "<NULL>".to_string(),
        _ => value.to_string(),
    }
}

fn any_value_to_db_string(value: &AnyValue<'_>) -> Option<String> {
    match value {
        AnyValue::Null => None,
        _ => Some(value.to_string()),
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

fn create_table_with_text_columns(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
) -> Result<(), String> {
    let column_definitions = merged_headers
        .iter()
        .map(|header| format!("{} TEXT", quote_identifier(header)))
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
                parts.push(value_ref_to_key_part(value));
            }

            keys.insert(parts.join("\u{1f}"));
        }

        Ok(keys)
    })
}

fn insert_non_duplicate_rows(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    merged_df: &polars::prelude::DataFrame,
    existing_keys: &mut HashSet<String>,
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
            let row_key = dataframe_row_to_key(&row.0);

            if existing_keys.contains(&row_key) {
                rows_skipped_duplicates += 1;
                continue;
            }

            let values = row
                .0
                .iter()
                .map(any_value_to_db_string)
                .collect::<Vec<_>>();

            stmt.execute(params_from_iter(values.iter()))
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

fn dataframe_rows_to_keyset(df: &polars::prelude::DataFrame) -> Result<HashSet<String>, String> {
    let mut keys = HashSet::with_capacity(df.height());

    for row_index in 0..df.height() {
        let row = df
            .get_row(row_index)
            .map_err(|e| format!("Failed to read merged DataFrame row: {e}"))?;

        let row_key = row
            .0
            .iter()
            .map(any_value_to_key_part)
            .collect::<Vec<_>>()
            .join("\u{1f}");

        keys.insert(row_key);
    }

    Ok(keys)
}

fn count_duplicates_with_table(
    db_state: &DuckDbState,
    table_name: &str,
    merged_headers: &[String],
    merged_df: &polars::prelude::DataFrame,
) -> Result<usize, String> {
    let merged_keys = dataframe_rows_to_keyset(merged_df)?;
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
                parts.push(value_ref_to_key_part(value));
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

    let table_name = suggested_table_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_identifier)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| auto_table_name(&merge.merged_headers));

    ensure_table_does_not_exist(db_state, &table_name)?;
    create_table_with_text_columns(db_state, &table_name, &merge.merged_headers)?;

    let mut existing_keys = HashSet::new();
    let (rows_inserted, rows_skipped_duplicates) = insert_non_duplicate_rows(
        db_state,
        &table_name,
        &merge.merged_headers,
        &merge.deduplicated_df,
        &mut existing_keys,
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

    let mut existing_keys = load_existing_row_keys(db_state, &table_name, &merge.merged_headers)?;
    let (rows_inserted, rows_skipped_duplicates) = insert_non_duplicate_rows(
        db_state,
        &table_name,
        &merge.merged_headers,
        &merge.deduplicated_df,
        &mut existing_keys,
    )?;

    Ok(models::CsvIngestionWriteResult {
        table_name,
        input_rows: merge.input_rows,
        rows_inserted,
        rows_skipped_duplicates,
        created_new_table: false,
    })
}