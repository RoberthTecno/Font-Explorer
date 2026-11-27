use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    System,
    User,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::System => "system",
            Source::User => "user",
        }
    }
    pub fn parse(s: &str) -> Source {
        if s == "user" {
            Source::User
        } else {
            Source::System
        }
    }
}

/// Resultado guardado de un archivo ya analizado; permite no volver a leerlo si no cambió.
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub size: u64,
    pub modified_at: Option<i64>,
    pub faces: Vec<(String, FaceInfo)>,
}

/// Archivo candidato encontrado por el descubrimiento, aún sin leer.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisInfo {
    pub tag: String,
    pub min_value: f32,
    pub default_value: f32,
    pub max_value: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharSetDef {
    pub id: String,
    pub label: String,
    /// "latin" | "spanish" | "symbols"
    pub category: String,
    pub chars: String,
}

/// Una cara tipográfica dentro de un archivo (los .ttc/.otc pueden traer varias).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceInfo {
    pub face_index: u32,
    pub subfamily_name: String,
    pub full_name: Option<String>,
    pub weight_class: u16,
    /// Ancho OS/2 (1 = ultra condensada … 5 = normal … 9 = ultra expandida).
    pub width_class: u16,
    pub is_italic: bool,
    pub is_oblique: bool,
    pub is_variable: bool,
    /// Ejes de la tabla `fvar` (vacío si no es variable).
    pub axes: Vec<AxisInfo>,
    pub glyph_count: u16,
    /// Caracteres de `CharSetDef` que esta cara NO tiene (cadena vacía = los tiene todos).
    pub missing: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyFile {
    pub path: String,
    pub file_name: String,
    pub format: String,
    pub size: u64,
    pub modified_at: Option<i64>,
    pub source: Source,
    /// Solo las caras de este archivo que pertenecen a la familia.
    pub faces: Vec<FaceInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Family {
    /// Clave estable: nombre de familia en minúsculas.
    pub id: String,
    pub family_name: String,
    pub is_variable: bool,
    pub variant_count: usize,
    pub files: Vec<FamilyFile>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorKind {
    Missing,
    Unreadable,
    Invalid,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Missing => "missing",
            ErrorKind::Unreadable => "unreadable",
            ErrorKind::Invalid => "invalid",
        }
    }
    pub fn parse(s: &str) -> ErrorKind {
        match s {
            "missing" => ErrorKind::Missing,
            "unreadable" => ErrorKind::Unreadable,
            _ => ErrorKind::Invalid,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub path: String,
    pub file_name: String,
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub char_sets: Vec<CharSetDef>,
    pub families: Vec<Family>,
    pub errors: Vec<ScanError>,
    pub files_total: usize,
    pub files_ok: usize,
    /// Archivos con formato no compatible (p. ej. .fon); no cuentan como error.
    pub files_skipped: usize,
    /// Archivos cuyo resultado se reutilizó de la base de datos (ruta, tamaño y fecha iguales).
    pub files_reused: usize,
    pub cancelled: bool,
    pub scanned_at: i64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub processed: usize,
    pub total: usize,
    pub current_file: String,
    pub errors: usize,
}
