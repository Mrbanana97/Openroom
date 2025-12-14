mod cache;
mod commands;
mod gpu;
mod image_io;
mod metadata;
mod models;
mod recipe_io;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::open_folder,
            commands::get_thumbnail,
            commands::render_preview,
            commands::read_metadata,
            commands::save_recipe,
            commands::load_recipe,
            commands::detect_gpus
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
