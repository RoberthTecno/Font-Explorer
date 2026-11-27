//! Lee los candidatos, agrupa por familia y registra errores sin detener el escaneo.
use super::{charsets, discover, metadata, types::*};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Protección ante archivos anormalmente grandes (se leen enteros en memoria).
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

fn unix_secs(t: SystemTime) -> Option<i64> {
    t.duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

pub fn scan(
    candidates: Vec<Candidate>,
    cancel: &AtomicBool,
    cache: &HashMap<String, CachedFile>,
    mut on_progress: impl FnMut(&ScanProgress),
) -> ScanResult {
    let started = Instant::now();
    let unique = discover::dedupe(candidates);
    let total = unique.len();

    let mut families: HashMap<String, Family> = HashMap::new();
    let mut errors: Vec<ScanError> = Vec::new();
    let (mut ok, mut skipped, mut reused, mut cancelled) = (0usize, 0usize, 0usize, false);
    let mut last_emit = Instant::now() - Duration::from_secs(1);

    for (i, cand) in unique.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        let file_name = cand
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = cand.path.to_string_lossy().to_string();

        if last_emit.elapsed() >= Duration::from_millis(40) {
            on_progress(&ScanProgress {
                processed: i,
                total,
                current_file: file_name.clone(),
                errors: errors.len(),
            });
            last_emit = Instant::now();
        }

        let mut fail = |kind, message: String| {
            errors.push(ScanError {
                path: path_str.clone(),
                file_name: file_name.clone(),
                kind,
                message,
            });
        };

        if !discover::is_supported(&cand.path) {
            skipped += 1;
            continue;
        }
        let meta = match std::fs::metadata(&cand.path) {
            Ok(m) if m.is_file() => m,
            Ok(_) => {
                fail(ErrorKind::Unreadable, "no es un archivo".into());
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fail(
                    ErrorKind::Missing,
                    "el archivo registrado ya no existe".into(),
                );
                continue;
            }
            Err(e) => {
                fail(ErrorKind::Unreadable, e.to_string());
                continue;
            }
        };
        if meta.len() > MAX_FILE_BYTES {
            fail(ErrorKind::Unreadable, "archivo demasiado grande".into());
            continue;
        }
        let modified = meta.modified().ok().and_then(unix_secs);
        let cached = cache
            .get(&path_str)
            .filter(|c| modified.is_some() && c.size == meta.len() && c.modified_at == modified);
        let faces = match cached {
            Some(c) => {
                reused += 1;
                c.faces.clone()
            }
            None => {
                let data = match std::fs::read(&cand.path) {
                    Ok(d) => d,
                    Err(e) => {
                        fail(ErrorKind::Unreadable, e.to_string());
                        continue;
                    }
                };
                let stem = cand
                    .path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                match metadata::parse_font_data(&data, &stem) {
                    Ok(f) => f,
                    Err(e) => {
                        fail(ErrorKind::Invalid, e);
                        continue;
                    }
                }
            }
        };
        ok += 1;

        let format = discover::extension_lower(&cand.path)
            .unwrap_or_default()
            .to_uppercase();
        for (family_name, face) in faces {
            let id = family_name.trim().to_lowercase();
            let fam = families.entry(id.clone()).or_insert_with(|| Family {
                id,
                family_name: family_name.trim().to_string(),
                is_variable: false,
                variant_count: 0,
                files: Vec::new(),
            });
            fam.is_variable |= face.is_variable;
            fam.variant_count += 1;
            match fam.files.iter_mut().find(|f| f.path == path_str) {
                Some(file) => file.faces.push(face),
                None => fam.files.push(FamilyFile {
                    path: path_str.clone(),
                    file_name: file_name.clone(),
                    format: format.clone(),
                    size: meta.len(),
                    modified_at: modified,
                    source: cand.source,
                    faces: vec![face],
                }),
            }
        }
    }

    let mut families: Vec<Family> = families.into_values().collect();
    families.sort_by(|a, b| a.id.cmp(&b.id));
    for f in &mut families {
        f.files
            .sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    }

    let result = ScanResult {
        char_sets: charsets::defs(),
        families,
        errors,
        files_total: total,
        files_ok: ok,
        files_skipped: skipped,
        files_reused: reused,
        cancelled,
        scanned_at: unix_secs(SystemTime::now()).unwrap_or(0),
        duration_ms: started.elapsed().as_millis() as u64,
    };
    on_progress(&ScanProgress {
        processed: if cancelled {
            ok + skipped + result.errors.len()
        } else {
            total
        },
        total,
        current_file: String::new(),
        errors: result.errors.len(),
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cand(p: PathBuf) -> Candidate {
        Candidate {
            path: p,
            source: Source::System,
        }
    }

    #[test]
    fn errors_do_not_stop_the_scan_and_duplicates_are_ignored() {
        let dir = std::env::temp_dir().join("fx_scan_test");
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("broken.ttf");
        std::fs::write(&bad, b"not a font").unwrap();
        let missing = dir.join("gone.ttf");
        let other = dir.join("legacy.fon");
        std::fs::write(&other, b"x").unwrap();

        let cancel = AtomicBool::new(false);
        let r = scan(
            vec![cand(bad.clone()), cand(bad), cand(missing), cand(other)],
            &cancel,
            &HashMap::new(),
            |_| {},
        );
        assert_eq!(r.files_total, 3, "el duplicado se cuenta una sola vez");
        assert_eq!(r.errors.len(), 2);
        assert_eq!(r.files_skipped, 1);
        assert!(r.families.is_empty());
    }

    #[test]
    fn cancel_stops_early() {
        let cancel = AtomicBool::new(true);
        let r = scan(
            vec![cand(PathBuf::from("/nope/a.ttf"))],
            &cancel,
            &HashMap::new(),
            |_| {},
        );
        assert!(r.cancelled);
        assert!(r.errors.is_empty());
    }

    /// Escanea las fuentes reales del equipo donde se ejecute la prueba (si las hay).
    #[test]
    fn scans_real_system_fonts_when_available() {
        fn walk(d: &std::path::Path, out: &mut Vec<Candidate>) {
            let Ok(rd) = std::fs::read_dir(d) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out)
                } else if discover::is_supported(&p) {
                    out.push(cand(p))
                }
            }
        }
        let mut c = Vec::new();
        walk(std::path::Path::new("/usr/share/fonts"), &mut c);
        if c.is_empty() {
            return;
        }
        let cancel = AtomicBool::new(false);
        let r = scan(c, &cancel, &HashMap::new(), |_| {});
        eprintln!(
            "familias={} archivos_ok={} errores={}",
            r.families.len(),
            r.files_ok,
            r.errors.len()
        );
        for f in r.families.iter().take(6) {
            eprintln!(
                "  {} · {} caras · variable={} · {}",
                f.family_name,
                f.variant_count,
                f.is_variable,
                f.files
                    .iter()
                    .flat_map(|x| x
                        .faces
                        .iter()
                        .map(|g| format!("{}/{}", g.subfamily_name, g.weight_class)))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        assert!(r.files_ok > 0);
        // Verificación de cobertura con una fuente conocida (si está instalada).
        if let Some(f) = r.families.iter().find(|f| f.family_name == "DejaVu Sans") {
            let face = &f.files[0].faces[0];
            assert!(
                !face.missing.contains('ñ') && !face.missing.contains('Á'),
                "DejaVu Sans tiene ñ y Á"
            );
            assert!(face.glyph_count > 1000);
            eprintln!(
                "DejaVu Sans {} faltan: {:?}",
                face.subfamily_name, face.missing
            );
        }
        // Una fuente puede carecer de símbolos: debe quedar registrado, no asumido.
        let with_gaps = r
            .families
            .iter()
            .flat_map(|f| f.files.iter().flat_map(|x| x.faces.iter()))
            .filter(|g| !g.missing.is_empty())
            .count();
        eprintln!("caras con algún carácter faltante: {with_gaps}");
        let axes = r
            .families
            .iter()
            .flat_map(|f| f.files.iter().flat_map(|x| x.faces.iter()))
            .filter(|g| g.is_variable)
            .count();
        eprintln!("caras variables: {axes}");
        let n: usize = r.families.iter().map(|f| f.files.len()).sum();
        assert!(n >= r.families.len());
    }
}
