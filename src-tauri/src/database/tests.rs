use super::*;
use crate::fonts::{discover, scanner, types::*};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

fn face(missing: &str) -> FaceInfo {
    FaceInfo {
        face_index: 0,
        subfamily_name: "Regular".into(),
        full_name: None,
        weight_class: 400,
        width_class: 3,
        is_italic: false,
        is_oblique: false,
        is_variable: true,
        glyph_count: 500,
        missing: missing.into(),
        axes: vec![AxisInfo {
            tag: "wght".into(),
            min_value: 100.0,
            default_value: 400.0,
            max_value: 900.0,
        }],
    }
}

fn result(names: &[&str]) -> ScanResult {
    let families = names
        .iter()
        .map(|n| Family {
            id: n.to_lowercase(),
            family_name: n.to_string(),
            is_variable: true,
            variant_count: 1,
            files: vec![FamilyFile {
                path: format!("/fonts/{n}.ttf"),
                file_name: format!("{n}.ttf"),
                format: "TTF".into(),
                size: 10,
                modified_at: Some(5),
                source: Source::System,
                faces: vec![face("ñ€")],
            }],
        })
        .collect();
    ScanResult {
        char_sets: vec![],
        families,
        errors: vec![ScanError {
            path: "/fonts/bad.ttf".into(),
            file_name: "bad.ttf".into(),
            kind: ErrorKind::Invalid,
            message: "x".into(),
        }],
        files_total: 3,
        files_ok: names.len(),
        files_skipped: 0,
        files_reused: 0,
        cancelled: false,
        scanned_at: 100,
        duration_ms: 7,
    }
}

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

#[test]
fn migrations_are_idempotent() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| {
        migrate(c)?;
        migrate(c)?;
        let v: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        assert_eq!(v, MIGRATIONS.len() as i64);
        let n: i64 = c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(n, 7);
        Ok::<_, rusqlite::Error>(())
    })
    .unwrap();
}

#[test]
fn scan_roundtrips_through_the_database() {
    let db = Db::open_in_memory().unwrap();
    let r = result(&["Alpha", "Beta"]);
    db.with(|c| scan_store::save_scan(c, &r)).unwrap();
    let back = db.with(|c| scan_store::load_scan(c)).unwrap().unwrap();
    assert_eq!(json(&back.families), json(&r.families));
    assert_eq!(
        (back.errors.len(), back.files_total, back.scanned_at),
        (1, 3, 100)
    );
}

#[test]
fn cancelled_scan_is_not_saved() {
    let db = Db::open_in_memory().unwrap();
    let mut r = result(&["Alpha"]);
    r.cancelled = true;
    db.with(|c| scan_store::save_scan(c, &r)).unwrap();
    assert!(db.with(|c| scan_store::load_scan(c)).unwrap().is_none());
}

#[test]
fn favorites_and_tags_survive_rescans_even_if_font_disappears() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha", "Beta"])))
        .unwrap();
    db.with(|c| -> Result<(), String> {
        user_store::set_favorite(c, "alpha", true)?;
        let t = user_store::create_tag(c, "Corporate", None)?;
        user_store::add_font_tag(c, "alpha", t.id)
    })
    .unwrap();
    // Alpha desaparece del equipo…
    db.with(|c| scan_store::save_scan(c, &result(&["Beta"])))
        .unwrap();
    let loaded = db.with(|c| scan_store::load_scan(c)).unwrap().unwrap();
    assert_eq!(loaded.families.len(), 1);
    // …y vuelve a instalarse: conserva favorito y etiqueta.
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha", "Beta"])))
        .unwrap();
    let ud = db.with(|c| user_store::user_data(c)).unwrap();
    assert_eq!(ud.favorites, vec!["alpha"]);
    assert_eq!(ud.font_tags.len(), 1);
    assert_eq!(
        db.with(|c| scan_store::load_scan(c))
            .unwrap()
            .unwrap()
            .families
            .len(),
        2
    );
}

#[test]
fn tag_rules() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha"])))
        .unwrap();
    db.with(|c| -> Result<(), String> {
        let t = user_store::create_tag(c, "  Print ", None)?;
        assert_eq!(t.name, "Print");
        assert!(user_store::create_tag(c, "print", None)
            .unwrap_err()
            .contains("already exists"));
        assert!(user_store::create_tag(c, "   ", None).is_err());
        let u = user_store::create_tag(c, "Web", None)?;
        assert!(user_store::rename_tag(c, u.id, "PRINT").is_err());
        user_store::rename_tag(c, u.id, "Web fonts")?;
        user_store::add_font_tag(c, "alpha", t.id)?;
        user_store::add_font_tag(c, "alpha", t.id)?; // repetir no duplica
        assert_eq!(user_store::user_data(c)?.font_tags.len(), 1);
        assert!(user_store::add_font_tag(c, "nope", t.id).is_err());
        user_store::delete_tag(c, t.id)?;
        let ud = user_store::user_data(c)?;
        assert!(ud.font_tags.is_empty());
        assert_eq!(ud.tags.len(), 1);
        Ok(())
    })
    .unwrap();
}

#[test]
fn settings_and_recents() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha", "Beta"])))
        .unwrap();
    db.with(|c| -> Result<(), String> {
        user_store::set_setting(c, "preview_size", "48")?;
        user_store::set_setting(c, "preview_size", "52")?;
        assert!(user_store::set_setting(c, "evil", "1").is_err());
        assert_eq!(user_store::user_data(c)?.settings["preview_size"], "52");
        user_store::mark_opened(c, "alpha")?;
        user_store::mark_opened(c, "beta")?;
        assert_eq!(user_store::user_data(c)?.recents, vec!["beta", "alpha"]);
        assert!(user_store::set_favorite(c, "ghost", true).is_err());
        Ok(())
    })
    .unwrap();
}

#[test]
fn unchanged_files_are_reused_and_charset_change_invalidates() {
    fn walk(d: &std::path::Path, out: &mut Vec<Candidate>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out)
            } else if discover::is_supported(&p) {
                out.push(Candidate {
                    path: p,
                    source: Source::System,
                })
            }
        }
    }
    let mut c = Vec::new();
    walk(std::path::Path::new("/usr/share/fonts"), &mut c);
    if c.is_empty() {
        return;
    }
    c.truncate(40);
    let cancel = AtomicBool::new(false);
    let first = scanner::scan(c.clone(), &cancel, &HashMap::new(), |_| {});
    assert_eq!(first.files_reused, 0);

    let db = Db::open_in_memory().unwrap();
    db.with(|conn| scan_store::save_scan(conn, &first)).unwrap();
    let cache = db.with(|conn| scan_store::load_cache(conn)).unwrap();
    let second = scanner::scan(c, &cancel, &cache, |_| {});
    eprintln!("reutilizados {}/{}", second.files_reused, second.files_ok);
    assert_eq!(
        second.files_reused, second.files_ok,
        "ningún archivo debía releerse"
    );
    assert_eq!(json(&second.families), json(&first.families));

    db.with(|conn| put_setting(conn, "charset_version", "0"))
        .unwrap();
    assert!(db
        .with(|conn| scan_store::load_cache(conn))
        .unwrap()
        .is_empty());
}

#[test]
fn only_scanned_font_files_are_known() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha"])))
        .unwrap();
    let known = |p: &str| db.with(|c| scan_store::is_known_font_file(c, p)).unwrap();
    assert!(known("/fonts/Alpha.ttf"));
    assert!(!known("/fonts/bad.ttf"), "un archivo con error no se sirve");
    assert!(!known("/etc/passwd"));
    assert!(!known("/fonts/../etc/passwd"));
}

#[test]
fn export_lists_files_only_for_requested_present_families() {
    let db = Db::open_in_memory().unwrap();
    db.with(|c| scan_store::save_scan(c, &result(&["Alpha", "Beta", "Gamma"])))
        .unwrap();
    let ask = |ids: &[&str]| {
        db.with(|c| {
            scan_store::files_for_families(
                c,
                &ids.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            )
        })
        .unwrap()
    };
    assert_eq!(
        ask(&["alpha", "gamma"]),
        vec![
            ("Alpha".into(), "/fonts/Alpha.ttf".into()),
            ("Gamma".into(), "/fonts/Gamma.ttf".into())
        ]
    );
    assert!(ask(&["nope"]).is_empty());
    assert!(ask(&[]).is_empty());
    // Una familia que dejó de estar instalada no se exporta.
    db.with(|c| scan_store::save_scan(c, &result(&["Beta"])))
        .unwrap();
    assert!(ask(&["alpha"]).is_empty());
    let many: Vec<String> = (0..1200).map(|i| format!("x{i}")).collect();
    assert!(
        db.with(|c| scan_store::files_for_families(c, &many))
            .unwrap()
            .is_empty(),
        "más de 500 ids: se trocean"
    );
}
