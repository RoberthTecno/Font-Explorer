# FontExplorer

Versión 5.3 — Slider rápido, importar/exportar fuentes (sobre la 5.2: preview con archivo real, comparación) (Tauri 2 + Vue 3 + TypeScript + Rust).

## Requisitos (Windows 10/11)
- Node.js 20+
- Rust (https://rustup.rs), toolchain MSVC, y "Desarrollo para el escritorio con C++" (Build Tools de Visual Studio)
- WebView2

## Uso
    npm install
    npm run tauri dev
    cargo test --manifest-path src-tauri/Cargo.toml     # pruebas del escáner y del análisis
    npm test                                            # pruebas de la lógica en TypeScript

## Qué hace la Fase 2
- Lee el registro de Windows (HKLM y HKCU) y las carpetas `%WINDIR%\Fonts` y `%LOCALAPPDATA%\Microsoft\Windows\Fonts`.
- Resuelve rutas, elimina duplicados, detecta archivos faltantes o dañados y sigue con el resto.
- Agrupa por familia (nombre tipográfico, ID 16; si no existe, ID 1).
- Escanea en segundo plano con progreso y botón Cancelar; "Rescan" en la barra y en Settings.

## Qué añade la Fase 3
- Por cada cara: peso, cursiva/oblicua, ejes `fvar` y cobertura `cmap` de ~110 caracteres (A–Z, a–z, español y 6 grupos de símbolos; ver `src-tauri/src/fonts/charsets.rs`).
- Semáforo de las tarjetas: verde = tiene A–Z, a–z y todo el español; amarillo = parcial; anillo = ninguno; gris = no analizado. Los símbolos no afectan al color.
- Familia: basta con que al menos una cara cumpla (spec §8).
- Clic en una tarjeta abre el detalle: pesos, estilos, archivos y cobertura carácter por carácter por cara.

## Qué añade la Fase 4
- SQLite local (`rusqlite`, SQLite incluido en la app) en `%APPDATA%\com.fontexplorer.app\fontexplorer.db`, con migraciones por `PRAGMA user_version`.
- El último escaneo se guarda: la biblioteca aparece al abrir la app y se refresca en segundo plano. Los archivos con la misma ruta, tamaño y fecha de modificación no se vuelven a leer.
- Favoritos, recientes (se registran al abrir el detalle) y etiquetas (crear, renombrar, borrar, asignar varias a una fuente) persisten. Si una fuente se desinstala y vuelve, conserva sus favoritos y etiquetas.
- Preferencias guardadas: tamaño de preview, texto de preview y vista (cuadrícula/lista), editables en Settings.
- Si cambian los conjuntos de caracteres, sube `charsets::VERSION` y se reanaliza todo.

## Qué añade la Fase 5
- Botón **Filters**: peso (la familia debe tener todos los marcados), Italic, Oblique, A–Z, a–z, tildes, español completo, todos los símbolos, favoritos y etiquetas (deben estar todas las marcadas). Todo se combina con "y" y con la búsqueda por nombre y la sección activa. Una familia cumple un filtro de cobertura si al menos una de sus variantes lo cumple.
- Fila de filtros activos con "X de Y familias" y botón para quitar cada uno.
- **Clic derecho** en una tarjeta (o tecla Menú / Shift+F10): favorito, abrir detalle y marcar o quitar etiquetas; se puede crear una etiqueta nueva y asignarla ahí mismo.
- **Detalle completo**: preview con texto editable en cada variante (con su peso y cursiva), resumen de español y símbolos, nombres internos, archivos, etiquetas y cobertura por variante. Si hay filtros de cobertura activos, marca la variante que cumple.
- Las familias sin análisis (datos de ejemplo) quedan excluidas de los filtros de análisis, no se muestran como "incompatibles".
- Pruebas: `npm test` (14) y `cargo test` (15).

## Versión 5.3
- **Slider de tamaño rápido:** el tamaño es una sola variable CSS del contenedor y las tarjetas fuera de pantalla no se calculan (`content-visibility: auto`). Con 1500 fuentes simuladas, cada cambio de tamaño pasó de ~1100 ms a ~32 ms. Las fuentes reales solo se cargan para las tarjetas visibles.
- **Importar (botón Import):** eliges archivos .ttf, .otf, .ttc o .otc, confirmas y se instalan **para tu usuario de Windows** (sin administrador): el archivo se **copia** a `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, se registra en `HKCU\...\CurrentVersion\Fonts` y se avisa a Windows (`AddFontResource` + `WM_FONTCHANGE`). El original no se mueve ni se borra. Solo se instala lo que se lee como fuente válida; si ya existe un archivo con ese nombre se omite. Después se vuelve a escanear solo. Requiere Windows 10 1809 o superior.
- **Exportar (botón Export, o Export en la barra de selección):** copia los archivos de las fuentes **mostradas ahora** (con filtros, etiquetas y búsqueda) o de las **seleccionadas** con el círculo, a la carpeta que elijas, en una subcarpeta por familia. Nunca sobrescribe.
- Seguridad: las rutas a exportar salen de la base de datos (no del frontend) y los nombres de carpeta se limpian.
- Pruebas: `npm test` (29) y `cargo test` (21).

## Versión 5.2
- **Preview con el archivo real de cada variante:** el backend entrega los bytes de la fuente (`read_font_file`, solo de archivos que el escáner registró) y el frontend los carga como `FontFace`. Así Ultra Bold, Black, etc. se dibujan con su archivo y no con el nombre de familia. Las tarjetas usan la variante representativa (Regular o la más cercana) y cargan de forma perezosa, con tope de 12 MB por archivo y un máximo de 300 fuentes cargadas a la vez. Si la carga falla, se usa familia + peso + estilo como respaldo. Limitación: dentro de un `.ttc/.otc` solo se carga la primera cara.
- **El detalle es una tercera columna** que empuja las tarjetas (en ventanas de menos de 1100 px vuelve a ser una capa).
- **Comparación:** círculo abajo a la derecha de cada tarjeta o Ctrl + clic (también "Add to comparison" en el clic derecho). De 2 a 6 fuentes, con texto y tamaño compartidos y variante por columna. Escape cierra.
- **Barra de escaneo:** la X cierra el aviso de errores (reaparece si un escaneo nuevo los produce); la lista completa está en Settings > View scan errors.
- **"Showing X fonts out of N":** siempre visible; N es toda la biblioteca y X tiene en cuenta sección, búsqueda, etiquetas, favoritos y filtros.
- Pruebas: `npm test` (27) y `cargo test` (16).

## Filtros inteligentes por variación
La barra lateral lista las variaciones que existen en tu biblioteca, con el número de familias de cada una:
- **Weight:** Thin, Extra/Ultra Light, Light, Regular, Medium, Semi/Demi Bold, Bold, Extra/Ultra Bold, Black/Heavy.
- **Style:** Italic, Oblique.
- **Width:** Condensed/Narrow, Expanded/Wide (clase de ancho OS/2, eje `wdth` o nombre de estilo de la propia variante).
Una familia aparece si al menos una de sus variantes la tiene. Se pueden combinar con el botón Filters. Los datos de ancho requieren reanalizar: la primera apertura tras actualizar relee todas las fuentes (la versión del análisis subió a 2).

## Limitaciones actuales
- Los filtros inteligentes de la barra lateral aún son fijos (no se pueden crear ni guardar).
