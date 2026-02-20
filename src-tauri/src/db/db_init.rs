use duckdb::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{App, AppHandle, Manager};

#[allow(dead_code)]
pub struct DuckDbState(pub Arc<Mutex<Connection>>);

fn resolve_db_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_local_data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data directory: {e}"))?;

    fs::create_dir_all(&app_local_data_dir)
        .map_err(|e| format!("Failed to create app local data directory: {e}"))?;

    Ok(app_local_data_dir.join("fidashy.duckdb"))
}

fn open_duckdb(path: &PathBuf) -> Result<Connection, String> {
    Connection::open(path).map_err(|e| format!("Failed to open DuckDB: {e}"))
}

fn initialize_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS app_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )
        .map_err(|e| format!("Failed to initialize DuckDB schema: {e}"))
}

pub fn setup_duckdb(app: &App) -> Result<(), String> {
    let db_path = resolve_db_path(&app.app_handle())?;
    let connection = open_duckdb(&db_path)?;
    initialize_schema(&connection)?;

    app.manage(DuckDbState(Arc::new(Mutex::new(connection))));
    println!("DuckDB initialized at {}", db_path.display());

    Ok(())
}