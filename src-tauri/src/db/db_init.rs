use duckdb::{params, Connection};
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

            CREATE TABLE IF NOT EXISTS greeted_people (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                greeted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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


// Sample Querries:

pub fn save_greeting(state: &DuckDbState, name: &str) -> Result<(), String> {
    let connection = state
        .0
        .lock()
        .map_err(|_| "Failed to acquire DuckDB lock".to_string())?;

    let next_id: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM greeted_people",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to compute next greeting id: {e}"))?;

    connection
        .execute(
            "INSERT INTO greeted_people (id, name) VALUES (?, ?)",
            params![next_id, name],
        )
        .map_err(|e| format!("Failed to save greeting: {e}"))?;

    Ok(()) 
}

pub fn list_greeted_people(state: &DuckDbState) -> Result<Vec<String>, String> {
    let connection = state
        .0
        .lock()
        .map_err(|_| "Failed to acquire DuckDB lock".to_string())?;

    let mut statement = connection
        .prepare("SELECT name FROM greeted_people ORDER BY greeted_at DESC")
        .map_err(|e| format!("Failed to prepare greeted people query: {e}"))?;

    let rows = statement
        .query_map([], |row| row.get::<usize, String>(0))
        .map_err(|e| format!("Failed to query greeted people: {e}"))?;

    let mut names = Vec::new();
    for row in rows {
        names.push(row.map_err(|e| format!("Failed to read greeted person row: {e}"))?);
    }

    Ok(names)
}