//! Comandos de datos del usuario: favoritos, etiquetas, recientes y preferencias.
use crate::database::{
    user_store,
    user_store::{Tag, UserData},
    Db,
};
use tauri::State;

#[tauri::command]
pub fn get_user_data(db: State<'_, Db>) -> Result<UserData, String> {
    db.with(|c| user_store::user_data(c))
}

#[tauri::command]
pub fn set_favorite(db: State<'_, Db>, font_id: String, value: bool) -> Result<(), String> {
    db.with(|c| user_store::set_favorite(c, &font_id, value))
}

#[tauri::command]
pub fn mark_opened(db: State<'_, Db>, font_id: String) -> Result<(), String> {
    db.with(|c| user_store::mark_opened(c, &font_id))
}

#[tauri::command]
pub fn create_tag(db: State<'_, Db>, name: String, color: Option<String>) -> Result<Tag, String> {
    db.with(|c| user_store::create_tag(c, &name, color.as_deref()))
}

#[tauri::command]
pub fn rename_tag(db: State<'_, Db>, id: i64, name: String) -> Result<(), String> {
    db.with(|c| user_store::rename_tag(c, id, &name))
}

#[tauri::command]
pub fn delete_tag(db: State<'_, Db>, id: i64) -> Result<(), String> {
    db.with(|c| user_store::delete_tag(c, id))
}

#[tauri::command]
pub fn add_font_tag(db: State<'_, Db>, font_id: String, tag_id: i64) -> Result<(), String> {
    db.with(|c| user_store::add_font_tag(c, &font_id, tag_id))
}

#[tauri::command]
pub fn remove_font_tag(db: State<'_, Db>, font_id: String, tag_id: i64) -> Result<(), String> {
    db.with(|c| user_store::remove_font_tag(c, &font_id, tag_id))
}

#[tauri::command]
pub fn set_setting(db: State<'_, Db>, key: String, value: String) -> Result<(), String> {
    db.with(|c| user_store::set_setting(c, &key, &value))
}
