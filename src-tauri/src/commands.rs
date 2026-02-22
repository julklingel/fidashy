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
            
        ]
    };
}