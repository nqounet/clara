pub mod cli;
pub mod commands;
pub mod config;
pub mod models;
pub mod parser;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::create_and_send_prompt,
            commands::get_app_config,
            commands::update_root_dir,
            commands::get_clara_config,
            commands::update_clara_config,
            commands::list_recent_atoms,
            commands::load_atom,
            commands::search_skr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
