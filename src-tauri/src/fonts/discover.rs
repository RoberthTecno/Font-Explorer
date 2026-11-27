//! Descubre archivos de fuentes instaladas: registro de Windows + carpetas estándar.
//! Solo devuelve rutas; no lee ni modifica ningún archivo de fuente.
use super::types::{Candidate, Source};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Formatos que se analizan. TTC/OTC se admiten porque varias fuentes de Windows vienen así.
pub const SUPPORTED_EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];

pub fn extension_lower(path: &Path) -> Option<String> {
    path.extension().map(|e| e.to_string_lossy().to_lowercase())
}

pub fn is_supported(path: &Path) -> bool {
    extension_lower(path).map_or(false, |e| SUPPORTED_EXTENSIONS.contains(&e.as_str()))
}

/// Los valores del registro pueden ser rutas absolutas o nombres relativos a la carpeta de fuentes.
pub fn resolve_registry_path(data: &str, fonts_dir: &Path) -> PathBuf {
    let p = PathBuf::from(data.trim());
    if p.is_absolute() {
        p
    } else {
        fonts_dir.join(p)
    }
}

/// Clave de deduplicación: ruta real si existe (sin prefijo `\\?\`) y sin distinguir mayúsculas.
pub fn dedup_key(path: &Path) -> String {
    let real = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let s = real.to_string_lossy().to_string();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
    if cfg!(windows) {
        s.to_lowercase()
    } else {
        s
    }
}

/// Elimina candidatos repetidos conservando el primero (el registro gana sobre la carpeta).
pub fn dedupe(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|c| seen.insert(dedup_key(&c.path)))
        .collect()
}

#[cfg(windows)]
fn registry_values(source: Source) -> Vec<String> {
    use winreg::{enums::*, RegKey};
    let hive = match source {
        Source::System => RegKey::predef(HKEY_LOCAL_MACHINE),
        Source::User => RegKey::predef(HKEY_CURRENT_USER),
    };
    let Ok(key) = hive.open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        KEY_READ,
    ) else {
        return Vec::new();
    };
    let names: Vec<String> = key
        .enum_values()
        .filter_map(|r| r.ok())
        .map(|(n, _)| n)
        .collect();
    names
        .iter()
        .filter_map(|n| key.get_value::<String, _>(n).ok())
        .collect()
}

#[cfg(not(windows))]
fn registry_values(_source: Source) -> Vec<String> {
    Vec::new()
}

fn list_dir(dir: &Path, source: Source) -> Vec<Candidate> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    rd.filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .map(|path| Candidate { path, source })
        .collect()
}

/// Registro (sistema y usuario) primero; después las carpetas estándar como complemento.
pub fn discover_installed() -> Vec<Candidate> {
    let fonts_dir = std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SystemRoot"))
        .map(|w| PathBuf::from(w).join("Fonts"));
    let user_dir = std::env::var_os("LOCALAPPDATA").map(|l| {
        PathBuf::from(l)
            .join("Microsoft")
            .join("Windows")
            .join("Fonts")
    });

    let mut out = Vec::new();
    for source in [Source::System, Source::User] {
        for value in registry_values(source) {
            // Sin carpeta base no se pueden resolver rutas relativas; se omiten.
            let base = fonts_dir.clone().unwrap_or_default();
            out.push(Candidate {
                path: resolve_registry_path(&value, &base),
                source,
            });
        }
    }
    if let Some(d) = &fonts_dir {
        out.extend(list_dir(d, Source::System));
    }
    if let Some(d) = &user_dir {
        out.extend(list_dir(d, Source::User));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_registry_values_resolve_against_fonts_dir() {
        let r = resolve_registry_path("arial.ttf", Path::new("/w/Fonts"));
        assert_eq!(r, PathBuf::from("/w/Fonts/arial.ttf"));
        let a = resolve_registry_path("/x/y.ttf", Path::new("/w/Fonts"));
        assert_eq!(a, PathBuf::from("/x/y.ttf"));
    }

    #[test]
    fn supported_formats() {
        assert!(is_supported(Path::new("A.TTF")));
        assert!(is_supported(Path::new("a.otc")));
        assert!(!is_supported(Path::new("a.fon")));
        assert!(!is_supported(Path::new("noext")));
    }

    #[test]
    fn dedupe_keeps_first_and_drops_repeats() {
        let c = |p: &str, s| Candidate {
            path: PathBuf::from(p),
            source: s,
        };
        let v = dedupe(vec![
            c("/nope/a.ttf", Source::System),
            c("/nope/a.ttf", Source::User),
            c("/nope/b.ttf", Source::User),
        ]);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].source, Source::System);
    }
}
