//! Guarda y recupera los resultados del escaneo (familias, archivos, caras, ejes).
use super::{get_setting, now, put_setting};
use crate::fonts::{charsets, types::*};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    scanned_at: i64,
    duration_ms: u64,
    files_total: usize,
    files_ok: usize,
    files_skipped: usize,
}

struct FaceRow {
    file_id: i64,
    family_id: String,
    family_name: String,
    present: bool,
    face: FaceInfo,
}

fn read_faces(conn: &Connection) -> rusqlite::Result<Vec<FaceRow>> {
    let mut axes: HashMap<i64, Vec<AxisInfo>> = HashMap::new();
    let mut st = conn.prepare("SELECT font_face_id, axis_tag, min_value, default_value, max_value FROM font_axes ORDER BY id")?;
    for row in st.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            AxisInfo {
                tag: r.get(1)?,
                min_value: r.get::<_, f64>(2)? as f32,
                default_value: r.get::<_, f64>(3)? as f32,
                max_value: r.get::<_, f64>(4)? as f32,
            },
        ))
    })? {
        let (id, a) = row?;
        axes.entry(id).or_default().push(a);
    }
    let mut st = conn.prepare(
        "SELECT fa.id, fa.font_file_id, fo.id, fo.family_name, fo.present, fa.face_index, fa.subfamily_name, fa.full_name,
                fa.weight_class, fa.is_italic, fa.is_oblique, fa.is_variable, fa.glyph_count, fa.missing_chars, fa.width_class
         FROM font_faces fa JOIN fonts fo ON fo.id = fa.font_id ORDER BY fa.id",
    )?;
    let rows = st.query_map([], |r| {
        let face_id: i64 = r.get(0)?;
        Ok(FaceRow {
            file_id: r.get(1)?,
            family_id: r.get(2)?,
            family_name: r.get(3)?,
            present: r.get::<_, i64>(4)? != 0,
            face: FaceInfo {
                face_index: r.get(5)?,
                subfamily_name: r.get(6)?,
                full_name: r.get(7)?,
                weight_class: r.get(8)?,
                width_class: r.get::<_, Option<i64>>(14)?.unwrap_or(5) as u16,
                is_italic: r.get::<_, i64>(9)? != 0,
                is_oblique: r.get::<_, i64>(10)? != 0,
                is_variable: r.get::<_, i64>(11)? != 0,
                glyph_count: r.get(12)?,
                missing: r.get(13)?,
                axes: axes.remove(&face_id).unwrap_or_default(),
            },
        })
    })?;
    rows.collect()
}

/// Archivos ya analizados, por ruta. Vacío si los conjuntos de caracteres cambiaron de versión.
pub fn load_cache(conn: &Connection) -> rusqlite::Result<HashMap<String, CachedFile>> {
    if get_setting(conn, "charset_version")?.as_deref() != Some(&charsets::VERSION.to_string()) {
        return Ok(HashMap::new());
    }
    let mut by_id: HashMap<i64, String> = HashMap::new();
    let mut out: HashMap<String, CachedFile> = HashMap::new();
    let mut st = conn.prepare(
        "SELECT id, file_path, file_size, modified_at FROM font_files WHERE scan_status = 'ok'",
    )?;
    for row in st.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, Option<i64>>(3)?,
        ))
    })? {
        let (id, path, size, modified_at) = row?;
        by_id.insert(id, path.clone());
        out.insert(
            path,
            CachedFile {
                size: size as u64,
                modified_at,
                faces: Vec::new(),
            },
        );
    }
    for fr in read_faces(conn)? {
        if let Some(c) = by_id.get(&fr.file_id).and_then(|p| out.get_mut(p)) {
            c.faces.push((fr.family_name, fr.face));
        }
    }
    Ok(out)
}

/// Guarda un escaneo completo. Un escaneo cancelado es parcial: no se guarda para no marcar
/// como "ausentes" las fuentes que simplemente no llegaron a leerse.
pub fn save_scan(conn: &mut Connection, r: &ScanResult) -> rusqlite::Result<()> {
    if r.cancelled {
        return Ok(());
    }
    let ts = now();
    let tx = conn.transaction()?;
    tx.execute("UPDATE fonts SET present = 0", [])?;
    tx.execute("DELETE FROM font_files", [])?; // arrastra caras y ejes (ON DELETE CASCADE)

    let mut file_ids: HashMap<String, i64> = HashMap::new();
    for fam in &r.families {
        for f in &fam.files {
            if file_ids.contains_key(&f.path) {
                continue;
            }
            tx.execute(
                "INSERT INTO font_files(file_path, file_name, format, file_size, modified_at, source, scan_status)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'ok')",
                params![f.path, f.file_name, f.format, f.size as i64, f.modified_at, f.source.as_str()],
            )?;
            file_ids.insert(f.path.clone(), tx.last_insert_rowid());
        }
    }
    for e in &r.errors {
        tx.execute(
            "INSERT OR IGNORE INTO font_files(file_path, file_name, format, file_size, source, scan_status, scan_error)
             VALUES(?1, ?2, '', 0, 'system', ?3, ?4)",
            params![e.path, e.file_name, e.kind.as_str(), e.message],
        )?;
    }
    for fam in &r.families {
        tx.execute(
            "INSERT INTO fonts(id, family_name, display_name, created_at, updated_at, last_scanned_at)
             VALUES(?1, ?2, ?2, ?3, ?3, ?3)
             ON CONFLICT(id) DO UPDATE SET family_name = ?2, present = 1, updated_at = ?3, last_scanned_at = ?3",
            params![fam.id, fam.family_name, ts],
        )?;
        for f in &fam.files {
            let file_id = file_ids[&f.path];
            for face in &f.faces {
                tx.execute(
                    "INSERT INTO font_faces(font_id, font_file_id, face_index, subfamily_name, full_name, weight_class,
                        is_italic, is_oblique, is_variable, glyph_count, missing_chars, width_class)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![fam.id, file_id, face.face_index, face.subfamily_name, face.full_name, face.weight_class,
                        face.is_italic, face.is_oblique, face.is_variable, face.glyph_count, face.missing, face.width_class],
                )?;
                let face_id = tx.last_insert_rowid();
                for a in &face.axes {
                    tx.execute(
                        "INSERT INTO font_axes(font_face_id, axis_tag, min_value, default_value, max_value) VALUES(?1, ?2, ?3, ?4, ?5)",
                        params![face_id, a.tag, a.min_value as f64, a.default_value as f64, a.max_value as f64],
                    )?;
                }
            }
        }
    }
    let summary = Summary {
        scanned_at: r.scanned_at,
        duration_ms: r.duration_ms,
        files_total: r.files_total,
        files_ok: r.files_ok,
        files_skipped: r.files_skipped,
    };
    put_setting(
        &tx,
        "last_scan",
        &serde_json::to_string(&summary).unwrap_or_default(),
    )?;
    put_setting(&tx, "charset_version", &charsets::VERSION.to_string())?;
    tx.commit()
}

/// Reconstruye el último escaneo guardado (None si nunca se escaneó).
pub fn load_scan(conn: &Connection) -> rusqlite::Result<Option<ScanResult>> {
    let Some(json) = get_setting(conn, "last_scan")? else {
        return Ok(None);
    };
    let Ok(sum) = serde_json::from_str::<Summary>(&json) else {
        return Ok(None);
    };

    let mut files: HashMap<i64, (String, String, String, u64, Option<i64>, Source)> =
        HashMap::new();
    let mut errors = Vec::new();
    let mut st = conn.prepare("SELECT id, file_path, file_name, format, file_size, modified_at, source, scan_status, scan_error FROM font_files")?;
    for row in st.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, Option<i64>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, Option<String>>(8)?,
        ))
    })? {
        let (id, path, name, format, size, modified, source, status, err) = row?;
        if status == "ok" {
            files.insert(
                id,
                (
                    path,
                    name,
                    format,
                    size as u64,
                    modified,
                    Source::parse(&source),
                ),
            );
        } else {
            errors.push(ScanError {
                path,
                file_name: name,
                kind: ErrorKind::parse(&status),
                message: err.unwrap_or_default(),
            });
        }
    }

    let mut families: HashMap<String, Family> = HashMap::new();
    for fr in read_faces(conn)? {
        let (Some((path, name, format, size, modified, source)), true) =
            (files.get(&fr.file_id), fr.present)
        else {
            continue;
        };
        let fam = families
            .entry(fr.family_id.clone())
            .or_insert_with(|| Family {
                id: fr.family_id.clone(),
                family_name: fr.family_name.clone(),
                is_variable: false,
                variant_count: 0,
                files: Vec::new(),
            });
        fam.is_variable |= fr.face.is_variable;
        fam.variant_count += 1;
        match fam.files.iter_mut().find(|f| &f.path == path) {
            Some(f) => f.faces.push(fr.face),
            None => fam.files.push(FamilyFile {
                path: path.clone(),
                file_name: name.clone(),
                format: format.clone(),
                size: *size,
                modified_at: *modified,
                source: *source,
                faces: vec![fr.face],
            }),
        }
    }
    let mut families: Vec<Family> = families.into_values().collect();
    families.sort_by(|a, b| a.id.cmp(&b.id));
    for f in &mut families {
        f.files
            .sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    }
    errors.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Some(ScanResult {
        char_sets: charsets::defs(),
        families,
        errors,
        files_total: sum.files_total,
        files_ok: sum.files_ok,
        files_skipped: sum.files_skipped,
        files_reused: 0,
        cancelled: false,
        scanned_at: sum.scanned_at,
        duration_ms: sum.duration_ms,
    }))
}

/// ¿El escáner registró este archivo como fuente válida? Evita servir rutas arbitrarias al frontend.
pub fn is_known_font_file(conn: &Connection, path: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM font_files WHERE file_path = ?1 AND scan_status = 'ok')",
        params![path],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n != 0)
}

/// Archivos (familia, ruta) de las familias indicadas, solo de fuentes presentes y válidas.
pub fn files_for_families(
    conn: &Connection,
    ids: &[String],
) -> rusqlite::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for chunk in ids.chunks(500) {
        let marks = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT DISTINCT fo.family_name, ff.file_path
             FROM font_faces fa JOIN fonts fo ON fo.id = fa.font_id JOIN font_files ff ON ff.id = fa.font_file_id
             WHERE fo.id IN ({marks}) AND fo.present = 1 AND ff.scan_status = 'ok'
             ORDER BY fo.family_name, ff.file_path"
        );
        let mut st = conn.prepare(&sql)?;
        let rows = st.query_map(rusqlite::params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            out.push(row?);
        }
    }
    Ok(out)
}
