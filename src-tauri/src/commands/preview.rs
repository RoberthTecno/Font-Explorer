use crate::database::{scan_store, Db};
use crate::fonts::discover;
use std::path::PathBuf;
use tauri::{ipc::Response, State};

/// Tope por archivo para previews (se envía entero al frontend).
const MAX_PREVIEW_BYTES: u64 = 64 * 1024 * 1024;

/// Entrega los bytes de una fuente para dibujar su preview con el archivo real de cada variante.
/// Solo sirve archivos que el escáner registró como fuentes válidas: nunca rutas arbitrarias.
/// Es `async` para que la lectura de archivos grandes no bloquee la ventana.
#[tauri::command]
pub async fn read_font_file(db: State<'_, Db>, path: String) -> Result<Response, String> {
    let known = db
        .with(|c| scan_store::is_known_font_file(c, &path))
        .map_err(|e| e.to_string())?;
    let p = PathBuf::from(&path);
    if !known || !discover::is_supported(&p) {
        return Err("Not a scanned font file".into());
    }
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let len = std::fs::metadata(&p).map_err(|e| e.to_string())?.len();
        if len > MAX_PREVIEW_BYTES {
            return Err("Font file is too large to preview".to_string());
        }
        std::fs::read(&p).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(Response::new(bytes))
}
