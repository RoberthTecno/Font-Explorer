//! Lectura de metadatos de una fuente. Es la única capa que conoce `ttf-parser`.
use super::{
    charsets,
    types::{AxisInfo, FaceInfo},
};
use ttf_parser::{name_id, Face, PlatformId};

/// Nombre `id` preferido en inglés de EE. UU. (Windows); si no existe, el primero decodificable.
fn name(face: &Face, id: u16) -> Option<String> {
    let mut fallback = None;
    for n in face.names() {
        if n.name_id != id {
            continue;
        }
        let Some(s) = n.to_string() else { continue };
        let s = s.trim().to_string();
        if s.is_empty() {
            continue;
        }
        if n.platform_id == PlatformId::Windows && n.language_id == 0x0409 {
            return Some(s);
        }
        fallback.get_or_insert(s);
    }
    fallback
}

/// Un glifo 0 (.notdef) cuenta como ausente.
fn has_glyph(face: &Face, c: char) -> bool {
    face.glyph_index(c).map_or(false, |g| g.0 != 0)
}

/// Familia con la que se agrupa: nombre tipográfico (ID 16) y, si no existe, el ID 1.
fn family_of(face: &Face) -> Option<String> {
    name(face, name_id::TYPOGRAPHIC_FAMILY).or_else(|| name(face, name_id::FAMILY))
}

/// Devuelve `(familia, cara)` por cada cara legible del archivo.
/// `fallback_family` se usa si la fuente no trae nombre de familia.
pub fn parse_font_data(
    data: &[u8],
    fallback_family: &str,
) -> Result<Vec<(String, FaceInfo)>, String> {
    let count = ttf_parser::fonts_in_collection(data).unwrap_or(1);
    let mut out = Vec::new();
    let mut last_err = None;
    for index in 0..count {
        match Face::parse(data, index) {
            Ok(face) => {
                let family = family_of(&face).unwrap_or_else(|| fallback_family.to_string());
                let subfamily = name(&face, name_id::TYPOGRAPHIC_SUBFAMILY)
                    .or_else(|| name(&face, name_id::SUBFAMILY))
                    .unwrap_or_else(|| "Regular".to_string());
                out.push((
                    family,
                    FaceInfo {
                        face_index: index,
                        subfamily_name: subfamily,
                        full_name: name(&face, name_id::FULL_NAME),
                        weight_class: face.weight().to_number(),
                        width_class: face.width().to_number(),
                        is_italic: face.is_italic(),
                        is_oblique: face.is_oblique(),
                        is_variable: face.is_variable(),
                        axes: face
                            .variation_axes()
                            .into_iter()
                            .map(|a| AxisInfo {
                                tag: String::from_utf8_lossy(&a.tag.to_bytes()).to_string(),
                                min_value: a.min_value,
                                default_value: a.def_value,
                                max_value: a.max_value,
                            })
                            .collect(),
                        glyph_count: face.number_of_glyphs(),
                        missing: charsets::missing_chars(|c| has_glyph(&face, c)),
                    },
                ));
            }
            Err(e) => last_err = Some(format!("cara {index}: {e:?}")),
        }
    }
    if out.is_empty() {
        Err(last_err.unwrap_or_else(|| "el archivo no contiene fuentes".to_string()))
    } else {
        Ok(out)
    }
}
