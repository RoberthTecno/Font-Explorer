//! Datos del usuario: favoritos, etiquetas, recientes y preferencias.
use super::{now, put_setting};
use rusqlite::{params, Connection, ErrorCode};
use serde::Serialize;
use std::collections::HashMap;

/// Preferencias que el frontend puede guardar.
const ALLOWED_SETTINGS: [&str; 3] = ["preview_size", "preview_text", "view_mode"];

#[derive(Serialize, Clone, Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontTag {
    pub font_id: String,
    pub tag_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub favorites: Vec<String>,
    pub tags: Vec<Tag>,
    pub font_tags: Vec<FontTag>,
    /// Ids de fuentes abiertas, la más reciente primero.
    pub recents: Vec<String>,
    pub settings: HashMap<String, String>,
}

fn db_err(e: rusqlite::Error) -> String {
    e.to_string()
}

pub fn user_data(conn: &Connection) -> Result<UserData, String> {
    let ids = |sql: &str| -> rusqlite::Result<Vec<String>> {
        conn.prepare(sql)?.query_map([], |r| r.get(0))?.collect()
    };
    let favorites = ids("SELECT id FROM fonts WHERE is_favorite = 1").map_err(db_err)?;
    let recents = ids("SELECT id FROM fonts WHERE last_opened_at IS NOT NULL AND present = 1 ORDER BY last_opened_at DESC, rowid DESC LIMIT 50").map_err(db_err)?;
    let tags = conn
        .prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")
        .and_then(|mut s| {
            s.query_map([], |r| {
                Ok(Tag {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                })
            })?
            .collect()
        })
        .map_err(db_err)?;
    let font_tags = conn
        .prepare("SELECT font_id, tag_id FROM font_tags")
        .and_then(|mut s| {
            s.query_map([], |r| {
                Ok(FontTag {
                    font_id: r.get(0)?,
                    tag_id: r.get(1)?,
                })
            })?
            .collect()
        })
        .map_err(db_err)?;
    let settings = conn
        .prepare("SELECT key, value FROM settings WHERE key IN ('preview_size','preview_text','view_mode')")
        .and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect())
        .map_err(db_err)?;
    Ok(UserData {
        favorites,
        tags,
        font_tags,
        recents,
        settings,
    })
}

pub fn set_favorite(conn: &Connection, font_id: &str, value: bool) -> Result<(), String> {
    let n = conn
        .execute(
            "UPDATE fonts SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3",
            params![value, now(), font_id],
        )
        .map_err(db_err)?;
    if n == 0 {
        Err("Font not found".into())
    } else {
        Ok(())
    }
}

pub fn mark_opened(conn: &Connection, font_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE fonts SET last_opened_at = ?1 WHERE id = ?2",
        params![now(), font_id],
    )
    .map_err(db_err)?;
    Ok(())
}

fn clean_name(name: &str) -> Result<&str, String> {
    let n = name.trim();
    if n.is_empty() {
        Err("Tag name can't be empty".into())
    } else {
        Ok(n)
    }
}

fn tag_err(e: rusqlite::Error) -> String {
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == ErrorCode::ConstraintViolation => {
            "A tag with that name already exists".into()
        }
        _ => e.to_string(),
    }
}

pub fn create_tag(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag, String> {
    let name = clean_name(name)?;
    conn.execute(
        "INSERT INTO tags(name, color, created_at) VALUES(?1, ?2, ?3)",
        params![name, color, now()],
    )
    .map_err(tag_err)?;
    Ok(Tag {
        id: conn.last_insert_rowid(),
        name: name.into(),
        color: color.map(String::from),
    })
}

pub fn rename_tag(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    let name = clean_name(name)?;
    let n = conn
        .execute("UPDATE tags SET name = ?1 WHERE id = ?2", params![name, id])
        .map_err(tag_err)?;
    if n == 0 {
        Err("Tag not found".into())
    } else {
        Ok(())
    }
}

pub fn delete_tag(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(db_err)?; // font_tags en cascada
    Ok(())
}

pub fn add_font_tag(conn: &Connection, font_id: &str, tag_id: i64) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO font_tags(font_id, tag_id) VALUES(?1, ?2)",
        params![font_id, tag_id],
    )
    .map_err(|e| match e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == ErrorCode::ConstraintViolation => {
            "Font or tag not found".to_string()
        }
        other => other.to_string(),
    })?;
    Ok(())
}

pub fn remove_font_tag(conn: &Connection, font_id: &str, tag_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM font_tags WHERE font_id = ?1 AND tag_id = ?2",
        params![font_id, tag_id],
    )
    .map_err(db_err)?;
    Ok(())
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    if !ALLOWED_SETTINGS.contains(&key) {
        return Err(format!("Unknown setting: {key}"));
    }
    put_setting(conn, key, value).map_err(db_err)
}
