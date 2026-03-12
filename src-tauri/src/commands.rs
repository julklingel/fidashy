/// # Command Registration Macro
///
/// This macro generates a Tauri command handler that registers all available
/// application commands. It provides a centralized way to manage and register
/// all command handlers across different modules.

#[macro_export]
macro_rules! get_commands {
    () => {
        tauri::generate_handler![
            // Sample
            $crate::sample::controller::greet,
            $crate::sample::controller::save_greeting,
            $crate::sample::controller::list_greeted_people,
            // CSV Ingestion
            $crate::csv_ingestion::controller::lazy_grouping_csv_many,
            $crate::csv_ingestion::controller::deduplicate_cached_group,
            $crate::csv_ingestion::controller::skip_merge_cached_group,
        
            $crate::csv_ingestion::controller::create_new_table_from_source,
            // $crate::csv_ingestion::controller::create_new_table_from_cached_group,
            $crate::csv_ingestion::controller::merge_source_file_into_table,
           //  $crate::csv_ingestion::controller::merge_cached_group_into_table,
            $crate::csv_ingestion::controller::find_groups_between_db_and_files,

       
        ]
    };
}