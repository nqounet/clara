pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod models;
pub mod parser;
pub mod search;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(search::SearchState::new())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::create_and_send_prompt,
            commands::get_app_config,
            commands::update_root_dir,
            commands::get_clara_config,
            commands::update_clara_config,
            commands::remove_workspace_history,
            commands::list_recent_atoms,
            commands::load_atom,
            commands::search_skr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
