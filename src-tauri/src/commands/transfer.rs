//! Importar fuentes (instalarlas para el usuario actual) y exportar fuentes de la biblioteca.
use crate::database::{scan_store, Db};
use crate::fonts::{transfer, winfonts};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    processed: usize,
    total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    done: usize,
    skipped: usize,
    failed: usize,
    items: Vec<transfer::ItemResult>,
    /// Carpeta de destino (la de fuentes del usuario al importar; la elegida al exportar).
    folder: String,
}

fn summarize(items: Vec<transfer::ItemResult>, folder: String) -> TransferResult {
    use transfer::ItemStatus::*;
    let count = |s| items.iter().filter(|i| i.status == s).count();
    TransferResult {
        done: count(Done),
        skipped: count(Skipped),
        failed: count(Error),
        items,
        folder,
    }
}

/// Copia las fuentes elegidas a la carpeta de fuentes del usuario y las registra en Windows.
/// No requiere administrador y no modifica ni mueve los archivos originales.
#[tauri::command]
pub async fn install_fonts(app: AppHandle, paths: Vec<String>) -> Result<TransferResult, String> {
    let dest = winfonts::user_fonts_dir().ok_or("Could not locate your fonts folder")?;
    let folder = dest.to_string_lossy().to_string();
    let items = tauri::async_runtime::spawn_blocking(move || {
        let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        transfer::install_files(
            &paths,
            &dest,
            &winfonts::register_user_font,
            |processed, total| {
                let _ = app.emit("transfer-progress", Progress { processed, total });
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(summarize(items, folder))
}

/// Copia a `destination/<familia>/` los archivos de las familias indicadas (los ids vienen de la
/// biblioteca; las rutas salen de la base de datos, nunca del frontend).
#[tauri::command]
pub async fn export_fonts(
    app: AppHandle,
    db: State<'_, Db>,
    font_ids: Vec<String>,
    destination: String,
) -> Result<TransferResult, String> {
    let dest = PathBuf::from(&destination);
    if !dest.is_dir() {
        return Err("The destination folder does not exist".into());
    }
    let files = db
        .with(|c| scan_store::files_for_families(c, &font_ids))
        .map_err(|e| e.to_string())?;
    if files.is_empty() {
        return Err("There are no fonts to export".into());
    }
    let items = tauri::async_runtime::spawn_blocking(move || {
        let list: Vec<(String, PathBuf)> = files
            .into_iter()
            .map(|(f, p)| (f, PathBuf::from(p)))
            .collect();
        transfer::export_files(&list, &dest, |processed, total| {
            let _ = app.emit("transfer-progress", Progress { processed, total });
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(summarize(items, destination))
}
