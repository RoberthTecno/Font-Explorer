//! Importar (instalar para el usuario) y exportar (copiar) archivos de fuentes.
//! La lógica no depende de Tauri ni de Windows: el registro de la fuente se inyecta como función.
use super::{discover, metadata, types::FaceInfo};
use serde::Serialize;
use std::path::{Path, PathBuf};

const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Done,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemResult {
    pub name: String,
    pub status: ItemStatus,
    pub message: String,
}

/// Registra una fuente ya copiada: (ruta destino, nombre del valor de registro).
pub type Register<'a> = &'a dyn Fn(&Path, &str) -> Result<(), String>;

/// Nombre seguro para una carpeta de Windows (sin caracteres inválidos ni nombres reservados).
pub fn sanitize_name(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_end_matches('.').trim().to_string();
    if cleaned.is_empty() {
        return "Font".into();
    }
    let base = cleaned.split('.').next().unwrap_or("").to_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&base.as_str())
        || ((base.starts_with("COM") || base.starts_with("LPT"))
            && base.len() == 4
            && base.chars().last().map_or(false, |c| c.is_ascii_digit()));
    if reserved {
        format!("_{cleaned}")
    } else {
        cleaned
    }
}

/// Nombre del valor de registro, p. ej. "Arial Bold (TrueType)"; en colecciones, hasta 3 nombres unidos por " & ".
fn registry_name(faces: &[(String, FaceInfo)], ext: &str) -> String {
    let names: Vec<String> = faces
        .iter()
        .take(3)
        .map(|(family, f)| {
            f.full_name
                .clone()
                .unwrap_or_else(|| format!("{family} {}", f.subfamily_name))
        })
        .collect();
    let kind = if ext == "otf" || ext == "otc" {
        "OpenType"
    } else {
        "TrueType"
    };
    format!("{} ({kind})", names.join(" & "))
}

fn file_name_of(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn install_one(src: &Path, dest_dir: &Path, register: Register) -> ItemResult {
    let name = file_name_of(src);
    let res = |status, message: String| ItemResult {
        name: name.clone(),
        status,
        message,
    };

    if !discover::is_supported(src) {
        return res(
            ItemStatus::Error,
            "Unsupported format (use .ttf, .otf, .ttc or .otc)".into(),
        );
    }
    let meta = match std::fs::metadata(src) {
        Ok(m) if m.is_file() => m,
        Ok(_) => return res(ItemStatus::Error, "Not a file".into()),
        Err(e) => return res(ItemStatus::Error, e.to_string()),
    };
    if meta.len() > MAX_FILE_BYTES {
        return res(ItemStatus::Error, "File is too large".into());
    }
    let data = match std::fs::read(src) {
        Ok(d) => d,
        Err(e) => return res(ItemStatus::Error, e.to_string()),
    };
    // Solo se instala lo que de verdad es una fuente legible.
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let faces = match metadata::parse_font_data(&data, &stem) {
        Ok(f) => f,
        Err(_) => return res(ItemStatus::Error, "Not a valid font file".into()),
    };
    let dest = dest_dir.join(&name);
    if dest.exists() {
        return res(
            ItemStatus::Skipped,
            "A font file with this name is already installed".into(),
        );
    }
    if let Err(e) = std::fs::create_dir_all(dest_dir).and_then(|_| std::fs::write(&dest, &data)) {
        return res(ItemStatus::Error, format!("Could not copy the file: {e}"));
    }
    let ext = discover::extension_lower(src).unwrap_or_default();
    if let Err(e) = register(&dest, &registry_name(&faces, &ext)) {
        let _ = std::fs::remove_file(&dest); // no se deja un archivo a medias sin registrar
        return res(
            ItemStatus::Error,
            format!("Could not register the font: {e}"),
        );
    }
    res(ItemStatus::Done, format!("Installed {}", faces[0].0))
}

pub fn install_files(
    paths: &[PathBuf],
    dest_dir: &Path,
    register: Register,
    mut progress: impl FnMut(usize, usize),
) -> Vec<ItemResult> {
    let total = paths.len();
    let mut out = Vec::with_capacity(total);
    for (i, src) in paths.iter().enumerate() {
        progress(i, total);
        out.push(install_one(src, dest_dir, register));
    }
    progress(total, total);
    out
}

/// Copia cada archivo a `dest/<familia>/<archivo>`. Nunca sobrescribe lo que ya existe.
pub fn export_files(
    items: &[(String, PathBuf)],
    dest: &Path,
    mut progress: impl FnMut(usize, usize),
) -> Vec<ItemResult> {
    let total = items.len();
    let mut out = Vec::with_capacity(total);
    for (i, (family, src)) in items.iter().enumerate() {
        progress(i, total);
        let name = file_name_of(src);
        let res = |status, message: String| ItemResult {
            name: name.clone(),
            status,
            message,
        };
        let dir = dest.join(sanitize_name(family));
        let target = dir.join(&name);
        out.push(if target.exists() {
            res(ItemStatus::Skipped, "Already exported".into())
        } else {
            match std::fs::create_dir_all(&dir).and_then(|_| std::fs::copy(src, &target)) {
                Ok(_) => res(ItemStatus::Done, format!("Copied to {}", dir.display())),
                Err(e) => res(ItemStatus::Error, e.to_string()),
            }
        });
    }
    progress(total, total);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn sample_font() -> Option<PathBuf> {
        fn walk(d: &Path) -> Option<PathBuf> {
            for e in std::fs::read_dir(d).ok()?.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Some(f) = walk(&p) {
                        return Some(f);
                    }
                } else if p.extension().map_or(false, |x| x == "ttf") {
                    return Some(p);
                }
            }
            None
        }
        walk(Path::new("/usr/share/fonts"))
    }
    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("fx_transfer_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn sanitizes_folder_names() {
        assert_eq!(sanitize_name("A:B/C?"), "A_B_C_");
        assert_eq!(sanitize_name("CON"), "_CON");
        assert_eq!(sanitize_name("com1"), "_com1");
        assert_eq!(sanitize_name("  Name. "), "Name");
        assert_eq!(sanitize_name("..."), "Font");
        assert_eq!(sanitize_name("Roboto Slab"), "Roboto Slab");
    }

    #[test]
    fn installs_valid_fonts_and_rejects_the_rest() {
        let Some(font) = sample_font() else { return };
        let src = tmp("src");
        let dest = tmp("dest").join("Fonts"); // aún no existe: debe crearse
        let good = src.join(font.file_name().unwrap());
        std::fs::copy(&font, &good).unwrap();
        let junk = src.join("broken.ttf");
        std::fs::write(&junk, b"not a font").unwrap();
        let text = src.join("notes.txt");
        std::fs::write(&text, b"x").unwrap();

        let registered = RefCell::new(Vec::<(PathBuf, String)>::new());
        let reg = |p: &Path, n: &str| {
            registered
                .borrow_mut()
                .push((p.to_path_buf(), n.to_string()));
            Ok(())
        };
        let r = install_files(
            &[good.clone(), junk, text, PathBuf::from("/nope/missing.ttf")],
            &dest,
            &reg,
            |_, _| {},
        );
        assert_eq!(r[0].status, ItemStatus::Done, "{:?}", r[0]);
        assert!(r[1].message.contains("valid font"));
        assert!(r[2].message.contains("Unsupported"));
        assert_eq!(r[3].status, ItemStatus::Error);
        assert!(dest.join(good.file_name().unwrap()).is_file());
        assert_eq!(
            registered.borrow().len(),
            1,
            "solo la fuente válida se registra"
        );
        assert!(registered.borrow()[0].1.ends_with("(TrueType)"));

        // Segunda vez: ya existe, no se duplica ni se vuelve a registrar.
        let again = install_files(&[good], &dest, &reg, |_, _| {});
        assert_eq!(again[0].status, ItemStatus::Skipped);
        assert_eq!(registered.borrow().len(), 1);
    }

    #[test]
    fn failed_registration_leaves_no_file_behind() {
        let Some(font) = sample_font() else { return };
        let dest = tmp("dest2");
        let reg = |_: &Path, _: &str| Err::<(), _>("denied".to_string());
        let r = install_files(&[font.clone()], &dest, &reg, |_, _| {});
        assert_eq!(r[0].status, ItemStatus::Error);
        assert!(r[0].message.contains("denied"));
        assert!(!dest.join(font.file_name().unwrap()).exists());
    }

    #[test]
    fn exports_into_family_folders_without_overwriting() {
        let Some(font) = sample_font() else { return };
        let dest = tmp("export");
        let items = vec![
            ("Bad:Name".to_string(), font.clone()),
            ("Other".to_string(), PathBuf::from("/nope/x.ttf")),
        ];
        let mut steps = Vec::new();
        let r = export_files(&items, &dest, |p, t| steps.push((p, t)));
        assert_eq!(r[0].status, ItemStatus::Done);
        assert!(dest
            .join("Bad_Name")
            .join(font.file_name().unwrap())
            .is_file());
        assert_eq!(r[1].status, ItemStatus::Error);
        assert_eq!(steps.last(), Some(&(2, 2)));
        let again = export_files(&items[..1], &dest, |_, _| {});
        assert_eq!(again[0].status, ItemStatus::Skipped);
    }
}
