use crate::csv_ingestion::models::{CreateTableFromSourceResult, MergeSourceIntoTableResult};
use crate::db::DuckDbState;

fn sql_quote_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('\"', "\"\""))
}

pub fn create_new_table_from_source(
    source_kind: String,
    source_id: String,
    source_paths: Vec<String>,
    preferred_table_name: String,
    db_state: &DuckDbState,
) -> Result<CreateTableFromSourceResult, String> {
    if source_paths.is_empty() {
        return Err("No source paths provided".to_string());
    }

    let quoted_table = sql_quote_ident(&preferred_table_name);

    let source_relation_sql = if source_paths.len() == 1 {
        let path = sql_quote_string(&source_paths[0]);
        format!("read_csv_auto({})", path)
    } else {
        let files = source_paths
            .iter()
            .map(|p| sql_quote_string(p))
            .collect::<Vec<_>>()
            .join(", ");

        format!("read_csv_auto([{}], union_by_name=true)", files)
    };

    let count_before_sql = format!("SELECT COUNT(*) FROM {}", source_relation_sql);
    let create_sql = format!(
        "CREATE TABLE {} AS SELECT DISTINCT * FROM {}",
        quoted_table, source_relation_sql
    );
    let count_after_sql = format!("SELECT COUNT(*) FROM {}", quoted_table);

    db_state.with_db(|conn| {
        let rows_before: i64 = conn
            .query_row(&count_before_sql, [], |row| row.get(0))
            .map_err(|e| format!("DuckDB Error (count before): {e}"))?;

        conn.execute(&create_sql, [])
            .map_err(|e| format!("DuckDB Error (create table): {e}"))?;

        let rows_after: i64 = conn
            .query_row(&count_after_sql, [], |row| row.get(0))
            .map_err(|e| format!("DuckDB Error (count after): {e}"))?;

        let rows_before = usize::try_from(rows_before)
            .map_err(|_| "Row count before dedup was negative".to_string())?;
        let rows_after = usize::try_from(rows_after)
            .map_err(|_| "Row count after dedup was negative".to_string())?;
        let duplicates_removed = rows_before.saturating_sub(rows_after);

        Ok(CreateTableFromSourceResult {
            source_kind: source_kind.clone(),
            source_id: source_id.clone(),
            target_table: preferred_table_name.clone(),
            rows_before,
            rows_after,
            duplicates_removed,
        })
    })
}

pub fn merge_source_into_table(
    source_kind: String,
    source_id: String,
    source_paths: Vec<String>,
    target_table: String,
    db_state: &DuckDbState,
) -> Result<MergeSourceIntoTableResult, String> {
    if source_paths.is_empty() {
        return Err("No source paths provided".to_string());
    }

    let quoted_table = sql_quote_ident(&target_table);

    let source_relation_sql = if source_paths.len() == 1 {
        let path = sql_quote_string(&source_paths[0]);
        format!("read_csv_auto({})", path)
    } else {
        let files = source_paths
            .iter()
            .map(|p| sql_quote_string(p))
            .collect::<Vec<_>>()
            .join(", ");
        format!("read_csv_auto([{}], union_by_name=true)", files)
    };

    // Count raw (pre-DISTINCT) rows from source — used to compute total duplicates eliminated.
    let count_source_raw_sql = format!("SELECT COUNT(*) FROM {}", source_relation_sql);

    // Count existing rows in the target before we insert anything.
    let count_before_sql = format!("SELECT COUNT(*) FROM {}", quoted_table);

    // Insert only rows that are new: distinct source rows minus rows already in the target.
    // EXCEPT removes cross-table duplicates; DISTINCT collapses intra-file duplicates first.
    let insert_sql = format!(
        "INSERT INTO {table} SELECT * FROM (SELECT DISTINCT * FROM {source} EXCEPT SELECT * FROM {table})",
        table = quoted_table,
        source = source_relation_sql,
    );

    let count_after_sql = format!("SELECT COUNT(*) FROM {}", quoted_table);

    db_state.with_db(|conn| {
        let rows_source_raw: i64 = conn
            .query_row(&count_source_raw_sql, [], |row| row.get(0))
            .map_err(|e| format!("DuckDB Error (count source rows): {e}"))?;

        let rows_before: i64 = conn
            .query_row(&count_before_sql, [], |row| row.get(0))
            .map_err(|e| format!("DuckDB Error (count before): {e}"))?;

        conn.execute(&insert_sql, [])
            .map_err(|e| format!("DuckDB Error (insert): {e}"))?;

        let rows_after: i64 = conn
            .query_row(&count_after_sql, [], |row| row.get(0))
            .map_err(|e| format!("DuckDB Error (count after): {e}"))?;

        let rows_source_raw = usize::try_from(rows_source_raw)
            .map_err(|_| "Source row count was negative".to_string())?;
        let rows_before = usize::try_from(rows_before)
            .map_err(|_| "Row count before merge was negative".to_string())?;
        let rows_after = usize::try_from(rows_after)
            .map_err(|_| "Row count after merge was negative".to_string())?;

        let rows_inserted = rows_after - rows_before;
        // Covers both intra-file duplicates and rows already present in the target table.
        let duplicates_removed = rows_source_raw.saturating_sub(rows_inserted);

        Ok(MergeSourceIntoTableResult {
            source_kind,
            source_id,
            target_table,
            rows_before,
            rows_after,
            rows_inserted,
            duplicates_removed,
        })
    })
}
