-- Esquema inicial. Ajuste respecto a la spec §7: font_id vive en font_faces (no en font_files),
-- porque un .ttc puede contener caras de familias distintas.
CREATE TABLE fonts (
    id              TEXT PRIMARY KEY,           -- clave estable: nombre de familia en minúsculas
    family_name     TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    present         INTEGER NOT NULL DEFAULT 1, -- 0 = ya no está instalada (se conservan favoritos y etiquetas)
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    last_scanned_at INTEGER,
    last_opened_at  INTEGER
);

CREATE TABLE font_files (
    id          INTEGER PRIMARY KEY,
    file_path   TEXT NOT NULL UNIQUE,
    file_name   TEXT NOT NULL,
    format      TEXT NOT NULL,
    file_size   INTEGER NOT NULL,
    modified_at INTEGER,
    source      TEXT NOT NULL,
    file_hash   TEXT,
    scan_status TEXT NOT NULL,                  -- ok | missing | unreadable | invalid
    scan_error  TEXT
);

CREATE TABLE font_faces (
    id             INTEGER PRIMARY KEY,
    font_id        TEXT NOT NULL REFERENCES fonts(id) ON DELETE CASCADE,
    font_file_id   INTEGER NOT NULL REFERENCES font_files(id) ON DELETE CASCADE,
    face_index     INTEGER NOT NULL,
    subfamily_name TEXT NOT NULL,
    full_name      TEXT,
    weight_class   INTEGER NOT NULL,
    width_class    INTEGER,
    is_italic      INTEGER NOT NULL,
    is_oblique     INTEGER NOT NULL,
    is_variable    INTEGER NOT NULL,
    glyph_count    INTEGER NOT NULL,
    missing_chars  TEXT NOT NULL                -- cobertura compacta: caracteres que FALTAN
);
CREATE INDEX idx_faces_font ON font_faces(font_id);
CREATE INDEX idx_faces_file ON font_faces(font_file_id);

CREATE TABLE font_axes (
    id            INTEGER PRIMARY KEY,
    font_face_id  INTEGER NOT NULL REFERENCES font_faces(id) ON DELETE CASCADE,
    axis_tag      TEXT NOT NULL,
    min_value     REAL NOT NULL,
    default_value REAL NOT NULL,
    max_value     REAL NOT NULL
);
CREATE INDEX idx_axes_face ON font_axes(font_face_id);

CREATE TABLE tags (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    color      TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE font_tags (
    font_id TEXT    NOT NULL REFERENCES fonts(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (font_id, tag_id)
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
