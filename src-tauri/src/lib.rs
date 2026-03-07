pub mod models;
pub mod core;

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
            core::create_and_send_prompt,
            core::get_app_config,
            core::update_root_dir,
            core::get_clara_config,
            core::update_clara_config,
            core::list_recent_atoms,
            core::load_atom
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
