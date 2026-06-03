#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            commands::register_quick_enhance_shortcut(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::enhance_prompt,
            commands::quick_enhance_clipboard,
            commands::list_templates,
            commands::apply_template,
            commands::get_config,
            commands::get_provider_setup_status,
            commands::save_settings,
            commands::set_api_key,
            commands::test_api_connection,
            commands::copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
