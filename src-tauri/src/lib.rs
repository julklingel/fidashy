mod db;

#[macro_use]
pub mod commands;
pub mod csv_ingestion;
pub mod sample;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(csv_ingestion::models::MergeCache::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    let builder = builder.setup(|app| {
        db::setup_duckdb(app).map_err(std::io::Error::other)?;
        Ok(())
    });

    builder
        .invoke_handler(get_commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
