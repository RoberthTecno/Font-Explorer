use crate::database::{scan_store, Db};
use crate::fonts::{discover, scanner, types::ScanResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
pub struct ScanState {
    running: AtomicBool,
    cancel: AtomicBool,
    last: Mutex<Option<ScanResult>>,
}

/// Libera la marca de "escaneando" aunque el hilo termine por un error inesperado.
struct RunningGuard(Arc<ScanState>);
impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
    }
}

/// Inicia el escaneo en un hilo aparte y vuelve de inmediato; la UI sigue respondiendo.
/// Progreso: evento `scan-progress`. Resultado final: evento `scan-done`.
#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<'_, Arc<ScanState>>) -> Result<(), String> {
    let state = state.inner().clone();
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Ya hay un escaneo en curso".into());
    }
    state.cancel.store(false, Ordering::SeqCst);

    std::thread::spawn(move || {
        let guard = RunningGuard(state.clone());
        let candidates = discover::discover_installed();
        let db = app.state::<Db>();
        // Resultados guardados: los archivos sin cambios (ruta, tamaño y fecha) no se vuelven a leer.
        let cache = db.with(|c| scan_store::load_cache(c)).unwrap_or_default();
        let result = scanner::scan(candidates, &state.cancel, &cache, |p| {
            let _ = app.emit("scan-progress", p);
        });
        if let Err(e) = db.with(|c| scan_store::save_scan(c, &result)) {
            eprintln!("could not save scan: {e}");
        }
        *state.last.lock().unwrap() = Some(result.clone());
        drop(guard);
        let _ = app.emit("scan-done", result);
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, Arc<ScanState>>) {
    state.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn get_last_scan(state: State<'_, Arc<ScanState>>, db: State<'_, Db>) -> Option<ScanResult> {
    if let Some(r) = state.last.lock().unwrap().clone() {
        return Some(r);
    }
    db.with(|c| scan_store::load_scan(c)).ok().flatten()
}

#[tauri::command]
pub fn is_scanning(state: State<'_, Arc<ScanState>>) -> bool {
    state.running.load(Ordering::SeqCst)
}
