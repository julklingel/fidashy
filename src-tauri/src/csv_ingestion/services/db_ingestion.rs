use crate::csv_ingestion::models::{
    CachedGroupDbMatch, FindGroupsBetweenDbAndFilesResult, FileDbCacheMatch,
     MergeCache, MergeCachedGroupIntoTableResult,
    MergeFileIntoTableResult,
};
use crate::db::DuckDbState;
use polars::prelude::{CsvWriter, SerWriter};
use std::collections::HashMap;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

fn quote_identifier(identifier: &str) -> Result<String, String> {
    let trimmed = identifier.trim();
    if trimmed.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }

    Ok(format!("\"{}\"", trimmed.replace('"', "\"\"")))
}

fn quote_sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn load_db_table_columns(
    db_state: &DuckDbState,
) -> Result<HashMap<String, Vec<String>>, String> {
    db_state.with_db(|db| {
        let mut stmt = db
            .prepare(
                "SELECT table_name, column_name
                 FROM information_schema.columns
                 WHERE table_schema = 'main'
                 ORDER BY table_name, ordinal_position",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;

        let mut db_table_map: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let (table_name, col_name) = row.map_err(|e| e.to_string())?;
            db_table_map.entry(table_name).or_default().push(col_name);
        }

        Ok(db_table_map)
    })
}

fn describe_file_columns(db_state: &DuckDbState, file_path: &str) -> Result<Vec<String>, String> {
    let file_literal = quote_sql_string(file_path);
    let sql = format!("DESCRIBE SELECT * FROM {file_literal}");

    db_state.with_db(|db| {
        let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
        let column_names = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(column_names)
    })
}


pub fn create_new_table_from_source(
    source_path: String,
    preferred_table_name: String,
    db_state: &DuckDbState,
) -> Result<(), String> {
    let table_identifier = quote_identifier(&preferred_table_name)?;
    let source_literal = quote_sql_string(&source_path);

    println!(
        "Creating table '{}' from source: {}",
        preferred_table_name, source_path
    );

    db_state.with_db(|conn| {
        let sql = format!("CREATE TABLE {table_identifier} AS SELECT * FROM {source_literal}");

        conn.execute(&sql, [])
            .map_err(|e| format!("DuckDB Error: {}", e))?;

        Ok(())
    })?;

    println!("Successfully created table '{}'", preferred_table_name);
    Ok(())
}

// pub fn create_new_table_from_cached_group(
//     group_id: String,
//     preferred_table_name: String,
//     cache: &MergeCache,
//     db_state: &DuckDbState,
// ) -> Result<CreateTableFromCachedGroupResult, String> {
//     // to do 
// }


pub fn merge_source_file_into_table(
    source_path: String,
    target_table: String,
    db_state: &DuckDbState,
) -> Result<MergeFileIntoTableResult, String> {
    let target_table_identifier = quote_identifier(&target_table)?;
    let source_literal = quote_sql_string(&source_path);
    let sql = format!(
        "INSERT INTO {target_table_identifier} SELECT * FROM {source_literal}"
    );

    let rows_written = db_state.with_db(|db| {
        db.execute(&sql, [])
            .map_err(|e| format!("DuckDB Error while merging '{}': {}", source_path, e))
    })?;

    Ok(MergeFileIntoTableResult {
        source_path: source_path.clone(),
        target_table: target_table.clone(),
        rows_written,
        message: format!(
            "Merged {} rows from '{}' into '{}'",
            rows_written, source_path, target_table
        ),
    })
}

// pub fn merge_cached_group_into_table(
//     group_id: String,
//     target_table: String,
//     cache: &MergeCache,
//     db_state: &DuckDbState,
// ) -> Result<MergeCachedGroupIntoTableResult, String> {
  
// }



pub fn find_groups_between_db_and_files(
    paths: Vec<String>,
    cache_ids: Vec<String>,
    cache: &MergeCache,
    db_state: &DuckDbState,
) -> Result<FindGroupsBetweenDbAndFilesResult, String> {
    let db_table_map = load_db_table_columns(db_state)?;
    let mut db_tables_by_columns: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    for (table_name, columns) in db_table_map {
        db_tables_by_columns
            .entry(columns)
            .or_default()
            .push(table_name);
    }

    let mut cache_groups_by_columns: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    let mut matched_groups: Vec<CachedGroupDbMatch> = Vec::new();
    for cache_id in cache_ids {
        if let Some(cached_df) = cache.get_group(&cache_id)? {
            let columns = cached_df
                .data_frame
                .get_column_names()
                .into_iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>();

            let matched_table_names = db_tables_by_columns
                .get(&columns)
                .cloned()
                .unwrap_or_default();

            if !matched_table_names.is_empty() {
                matched_groups.push(CachedGroupDbMatch {
                    group_id: cache_id.clone(),
                    paths: cached_df.paths.clone(),
                    matched_table_names,
                });
            }

            cache_groups_by_columns
                .entry(columns)
                .or_default()
                .push(cache_id);
        }
    }

    let mut matched_files: Vec<FileDbCacheMatch> = Vec::new();
    for file_path in paths {
        let file_columns = describe_file_columns(db_state, &file_path)?;
        let matched_table_names = db_tables_by_columns
            .get(&file_columns)
            .cloned()
            .unwrap_or_default();
        let matched_cache_group_ids = cache_groups_by_columns
            .get(&file_columns)
            .cloned()
            .unwrap_or_default();

        if !matched_table_names.is_empty() || !matched_cache_group_ids.is_empty() {
            matched_files.push(FileDbCacheMatch {
                file_path,
                matched_table_names,
                matched_cache_group_ids,
            });
        }
    }

    Ok(FindGroupsBetweenDbAndFilesResult {
        matched_files,
        matched_groups,
    })
}