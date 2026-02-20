mod db;

use tauri::State;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn save_greeting(name: &str, db_state: State<db::DuckDbState>) -> Result<(), String> {
    db::save_greeting(&db_state, name)
}

#[tauri::command]
fn list_greeted_people(db_state: State<db::DuckDbState>) -> Result<Vec<String>, String> {
    db::list_greeted_people(&db_state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    let builder = builder.setup(|app| {
        db::setup_duckdb(app).map_err(std::io::Error::other)?;
        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![greet, save_greeting, list_greeted_people])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
