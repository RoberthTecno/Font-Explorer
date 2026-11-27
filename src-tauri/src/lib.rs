mod commands;
mod database;
mod fonts;

use commands::{
    library, preview,
    scan::{self, ScanState},
    transfer,
};
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(ScanState::default()))
        .setup(|app| {
            // La base vive en la carpeta de datos de la app (en Windows: %APPDATA%\com.fontexplorer.app).
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(database::Db::open(&dir.join("fontexplorer.db"))?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan::start_scan,
            scan::cancel_scan,
            scan::get_last_scan,
            scan::is_scanning,
            library::get_user_data,
            library::set_favorite,
            library::mark_opened,
            library::create_tag,
            library::rename_tag,
            library::delete_tag,
            library::add_font_tag,
            library::remove_font_tag,
            library::set_setting,
            preview::read_font_file,
            transfer::install_fonts,
            transfer::export_fonts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FontExplorer");
}
