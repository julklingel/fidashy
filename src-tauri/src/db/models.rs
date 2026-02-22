use std::sync::{Arc, Mutex};
use duckdb::Connection;

#[allow(dead_code)]
#[derive(Clone)]
pub struct DuckDbState(pub Arc<Mutex<Connection>>);

impl DuckDbState {
    pub fn with_db<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let db = self
            .0
            .lock()
            .map_err(|_| "Failed to acquire DuckDB lock".to_string())?;

        f(&db)
    }
}

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"#;