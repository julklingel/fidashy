use polars::prelude::*;
use std::collections::HashMap;

use crate::csv_ingestion::models::{DbMatchProposal, GroupProposal};
use crate::db::DuckDbState;


pub fn lazy_grouping_csv_many(
    paths: Vec<String>,
) -> Result<Vec<GroupProposal>, String> {
    let mut groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

    for path in paths {
        let mut lf = LazyCsvReader::new(PlRefPath::from(path.as_str()))
            .with_has_header(true)
            .finish()
            .map_err(|e| format!("Failed to open CSV '{}': {e}", path))?;

        let mut columns = lf
            .collect_schema()
            .map_err(|e| format!("Failed to read CSV header '{}': {e}", path))?
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

    Ok(grouped_path_filtered
        .into_iter()
        .enumerate()
        .map(|(index, group_paths)| GroupProposal {
            group_id: format!("group-{}", index + 1),
            paths: group_paths,
        })
        .collect())
}



pub fn find_groups_between_db_and_files(
    groups: Vec<GroupProposal>,
    standalone_paths: Vec<String>,
    db_state: &DuckDbState,
) -> Result<Vec<DbMatchProposal>, String> {

    fn normalize_columns(mut columns: Vec<String>) -> Vec<String> {
        columns = columns
            .into_iter()
            .map(|column| column.to_lowercase())
            .collect();
        columns.sort();
        columns
    }

    fn read_csv_columns(db: &duckdb::Connection, path: &str) -> Result<Vec<String>, String> {
        let sql = format!("DESCRIBE SELECT * FROM '{}'", path);
        let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;

        let columns = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(normalize_columns(columns))
    }

    let mut db_tables_by_structure: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    let mut matches: Vec<DbMatchProposal> = Vec::new();

   
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
            db_tables_by_structure
                .entry(normalize_columns(cols))
                .or_default()
                .push(table_name);
        }

        for group in groups {
            if group.paths.is_empty() {
                continue;
            }

            let columns = read_csv_columns(db, &group.paths[0])?;
            let matching_tables = db_tables_by_structure
                .get(&columns)
                .cloned()
                .unwrap_or_default();

            matches.push(DbMatchProposal {
                source_kind: "group".to_string(),
                source_id: group.group_id,
                source_paths: group.paths,
                columns,
                matching_tables,
            });
        }

        for (index, path) in standalone_paths.into_iter().enumerate() {
            let columns = read_csv_columns(db, &path)?;
            let matching_tables = db_tables_by_structure
                .get(&columns)
                .cloned()
                .unwrap_or_default();

            matches.push(DbMatchProposal {
                source_kind: "standalone".to_string(),
                source_id: format!("standalone-{}", index + 1),
                source_paths: vec![path],
                columns,
                matching_tables,
            });
        }

        Ok(())
    })?;

    matches.sort_by(|left, right| left.source_id.cmp(&right.source_id));

    Ok(matches)
}