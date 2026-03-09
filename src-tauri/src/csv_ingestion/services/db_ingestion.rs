use crate::csv_ingestion::models::{DbImportActionResult, MergeCache};
use crate::db::DuckDbState;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy)]
pub enum ImportSourceKind {
    Group,
    File,
}

impl TryFrom<&str> for ImportSourceKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "group" => Ok(Self::Group),
            "file" => Ok(Self::File),
            other => Err(format!("Unsupported import source kind: {other}")),
        }
    }
}


pub fn create_new_table_from_source(
    _source_kind: ImportSourceKind,
    source_name: String,
    _source_paths: Vec<String>,
    preferred_table: String,
    _cache: &MergeCache,
    _db_state: &DuckDbState,
) -> Result<DbImportActionResult, String> {
    Ok(DbImportActionResult {
        target_table: preferred_table.clone(),
        rows_written: 0,
        source_label: source_name.clone(),
        message: format!(
            "Dummy create_new_table_from_source for '{source_name}' -> '{preferred_table}'"
        ),
    })
}

pub fn merge_source_into_table(
    _source_kind: ImportSourceKind,
    source_name: String,
    _source_paths: Vec<String>,
    target_table: String,
    _cache: &MergeCache,
    _db_state: &DuckDbState,
) -> Result<DbImportActionResult, String> {
    Ok(DbImportActionResult {
        target_table: target_table.clone(),
        rows_written: 0,
        source_label: source_name.clone(),
        message: format!(
            "Dummy merge_source_into_table for '{source_name}' -> '{target_table}'"
        ),
    })
}

pub fn find_groups_between_db_and_files(
    paths: Vec<String>,
    cache_ids: Vec<String>,
    cache: &MergeCache,
    db_state: &DuckDbState,
) -> Result<(), String> {
    // Key: Ordered Column Names | Value: List of sources (DB, File, or Cache)
    let mut structure_groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

    // --- 1. Process DuckDB Tables ---
    db_state.with_db(|db| {
        let mut stmt = db.prepare(
            "SELECT table_name, column_name 
             FROM information_schema.columns 
             WHERE table_schema = 'main' 
             ORDER BY table_name, ordinal_position",
        ).map_err(|e| e.to_string())?;

        let mut db_table_map: HashMap<String, Vec<String>> = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;

        for row in rows {
            let (table_name, col_name) = row.map_err(|e| e.to_string())?;
            db_table_map.entry(table_name).or_default().push(col_name);
        }

        for (table_name, cols) in db_table_map {
            structure_groups.entry(cols).or_default().push(format!("[DB] {}", table_name));
        }

        // --- 2. Process Files (via DuckDB DESCRIBE) ---
        for path in paths {
            // Using DESCRIBE is the most efficient way to peek at file schema in DuckDB
            let sql = format!("DESCRIBE SELECT * FROM '{}'", path);
            let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
            
            let file_cols: Vec<String> = stmt.query_map([], |row| {
                row.get::<_, String>(0) // The first col of DESCRIBE is the column name
            }).map_err(|e| e.to_string())?
              .collect::<Result<Vec<_>, _>>()
              .map_err(|e| e.to_string())?;

            structure_groups.entry(file_cols).or_default().push(format!("[FILE] {}", path));
        }

        Ok(())
    })?;

    // --- 3. Process Polars Cache ---
    for id in cache_ids {
        if let Some(cached_df) = cache.get_group(&id)? {
            // Get names from the Polars DataFrame
            let col_names: Vec<String> = cached_df.data_frame
                .get_column_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();

            structure_groups.entry(col_names).or_default().push(format!("[CACHE] {}", id));
        }
    }

    // --- 4. Output the results ---
    println!("--- Schema Grouping Results ---");
    for (structure, sources) in &structure_groups {
        if sources.len() > 1 {
            println!("\nMATCHING GROUP FOUND:");
            println!("Columns: {:?}", structure);
            for source in sources {
                println!("  -> {}", source);
            }
        }
    }

    Ok(())
}