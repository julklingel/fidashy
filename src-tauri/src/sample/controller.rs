
use super::services;
use crate::db::DuckDbState;
use tauri::State;


#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn save_greeting(name: &str, db_state: State<DuckDbState>) -> Result<(), String> {
    services::save_greeting(&db_state, name)
}

#[tauri::command]
pub fn list_greeted_people(db_state: State<DuckDbState>) -> Result<Vec<String>, String> {
    services::list_greeted_people(&db_state)
}