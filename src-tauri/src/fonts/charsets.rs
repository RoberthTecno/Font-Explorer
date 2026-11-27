//! Conjuntos de caracteres que se comprueban en cada fuente.
//! Las categorías `latin` y `spanish` definen "cumple el conjunto" (semáforo de las tarjetas);
//! `symbols` es informativo y alimenta el catálogo y los filtros.
use super::types::CharSetDef;

/// Súbela cuando cambien los conjuntos o los campos analizados: invalida los resultados guardados.
/// v2: se añadió el ancho (width_class) de cada cara.
pub const VERSION: u32 = 2;

pub struct CharSet {
    pub id: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub chars: &'static str,
}

pub const CHAR_SETS: &[CharSet] = &[
    CharSet {
        id: "upper",
        label: "Uppercase A–Z",
        category: "latin",
        chars: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    },
    CharSet {
        id: "lower",
        label: "Lowercase a–z",
        category: "latin",
        chars: "abcdefghijklmnopqrstuvwxyz",
    },
    CharSet {
        id: "accents",
        label: "Accented vowels",
        category: "spanish",
        chars: "áéíóúÁÉÍÓÚ",
    },
    CharSet {
        id: "enye",
        label: "Ñ / ñ",
        category: "spanish",
        chars: "ñÑ",
    },
    CharSet {
        id: "dieresis",
        label: "Ü / ü",
        category: "spanish",
        chars: "üÜ",
    },
    CharSet {
        id: "inverted",
        label: "Opening marks ¿ ¡",
        category: "spanish",
        chars: "¿¡",
    },
    CharSet {
        id: "currency",
        label: "Currency",
        category: "symbols",
        chars: "$¢£¥€",
    },
    CharSet {
        id: "legal",
        label: "Legal and text marks",
        category: "symbols",
        chars: "©®™§¶†‡",
    },
    CharSet {
        id: "punctuation",
        label: "Typographic punctuation",
        category: "symbols",
        chars: "…–—‘’“”«»•",
    },
    CharSet {
        id: "math",
        label: "Math and units",
        category: "symbols",
        chars: "±×÷−≠≤≥≈∞√°µ",
    },
    CharSet {
        id: "arrows",
        label: "Arrows",
        category: "symbols",
        chars: "←↑→↓↔",
    },
    CharSet {
        id: "misc",
        label: "Miscellaneous",
        category: "symbols",
        chars: "★☆♥✓",
    },
];

pub fn defs() -> Vec<CharSetDef> {
    CHAR_SETS
        .iter()
        .map(|s| CharSetDef {
            id: s.id.into(),
            label: s.label.into(),
            category: s.category.into(),
            chars: s.chars.into(),
        })
        .collect()
}

/// Caracteres de todos los conjuntos que la fuente NO tiene, concatenados en una cadena.
pub fn missing_chars(has_glyph: impl Fn(char) -> bool) -> String {
    CHAR_SETS
        .iter()
        .flat_map(|s| s.chars.chars())
        .filter(|c| !has_glyph(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn sets_are_disjoint_and_sized_as_expected() {
        let all: Vec<char> = CHAR_SETS.iter().flat_map(|s| s.chars.chars()).collect();
        let uniq: HashSet<_> = all.iter().collect();
        assert_eq!(
            all.len(),
            uniq.len(),
            "un carácter aparece en más de un conjunto"
        );
        let n = |id: &str| {
            CHAR_SETS
                .iter()
                .find(|s| s.id == id)
                .unwrap()
                .chars
                .chars()
                .count()
        };
        assert_eq!(
            (
                n("upper"),
                n("lower"),
                n("accents"),
                n("enye"),
                n("dieresis"),
                n("inverted")
            ),
            (26, 26, 10, 2, 2, 2)
        );
    }

    #[test]
    fn missing_chars_lists_only_absent_ones() {
        assert_eq!(missing_chars(|_| true), "");
        let m = missing_chars(|c| c != 'ñ' && c != '€');
        assert_eq!(m, "ñ€");
    }
}
