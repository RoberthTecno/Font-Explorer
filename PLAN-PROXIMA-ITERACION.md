# FontExplorer — Plan de la próxima iteración (5.2)

> **Estado: los 5 puntos se implementaron en la versión 5.2** (ver README). Pendiente de verificar en Windows con fuentes reales: el punto 1 (aspecto de variantes Ultra Bold) y el funcionamiento del comando `read_font_file` dentro de Tauri.

Pendientes pedidos tras la versión 5.1. Cada punto incluye cómo lo entiendo, cómo lo resolvería y cómo saber que está terminado. Los puntos marcados con **(confirmar)** tienen una duda que conviene aclarar antes de empezar.

Estado actual: Fases 1 a 5 y filtros inteligentes por variación (5.1) entregados. La **Fase 6 (calidad y distribución)** sigue pendiente y se retoma cuando se decida.

---

## ✅ 1. Corregir: las variantes Ultra Bold se ven como normales

**Qué pasa (según lo reportado):** en algunas fuentes, la variante Ultra Bold se ve como texto normal en las vistas pequeñas, donde se muestran las demás variantes.

**Hipótesis de la causa (por verificar):**
- Las tarjetas de la biblioteca dibujan el preview solo con el *nombre de la familia*, sin aplicar peso ni estilo. Si la fuente se instaló con otro nombre de familia para esa variante, el navegador no la encuentra y muestra la normal.
- Windows agrupa algunas fuentes antiguas en "familias" distintas por variante (p. ej. "Nombre Ultra Bold" como familia aparte). FontExplorer agrupa por nombre tipográfico, pero el navegador usa el nombre que conoce el sistema, y pueden no coincidir.

**Propuesta:**
1. Reproducir con una fuente afectada y ver si ocurre en la tarjeta, en la lista de variantes del detalle, o en ambas.
2. Dibujar cada variante con **el archivo real** de esa variante (cargarlo como `FontFace` desde un comando de Rust que lea el archivo validado), en vez de depender del nombre de familia. Así el preview es exacto aunque el nombre no coincida.
3. En las tarjetas, usar como preview la variante más representativa (Regular; si no existe, la más cercana a 400) aplicando su peso y estilo.

**Terminado cuando:** una familia con variante Ultra Bold muestra el peso correcto en el detalle y en la tarjeta, y no se confunde con Regular. Prueba automática del cálculo de la variante representativa.

**(confirmar)** Necesito el nombre de una fuente afectada y si se ve mal en las tarjetas, en las variantes del detalle o en ambos sitios.

---

## ✅ 2. El panel de detalle debe empujar las tarjetas, no taparlas

**Pedido:** al hacer clic en una fuente y abrirse la ventana de la derecha, los cuadros deben moverse para no quedar tapados.

**Hoy:** el detalle es una capa fija que cubre parte de la biblioteca.

**Propuesta:**
- Hacer del detalle una tercera columna del diseño principal. La biblioteca se estrecha y las tarjetas se reacomodan (la cuadrícula ya se adapta sola).
- La barra flotante de preview se centra en el área visible restante.
- En ventanas muy estrechas, volver al comportamiento de capa para no dejar la biblioteca ilegible.
- El panel de la izquierda ya reacomoda el contenido al abrirse y cerrarse; mantenerlo así.

**Terminado cuando:** al abrir el detalle ninguna tarjeta queda tapada y al cerrarlo la cuadrícula recupera su ancho. Revisar a 1280 px y a 900 px (ancho mínimo).

**(confirmar)** Entiendo que "toggle del lado" se refiere al panel derecho de detalle. Si también quieres cambiar algo del panel izquierdo, dime qué.

---

## ✅ 3. Modo comparación de fuentes

**Pedido:** seleccionar varias fuentes y verlas lado a lado con el mismo texto de prueba. La selección se hace con un **circulito abajo a la derecha de la tarjeta** o con **Ctrl + clic**.

**Propuesta:**
- **Selección:** círculo en la esquina inferior derecha de cada tarjeta (visible al pasar el cursor, relleno cuando está seleccionada) y Ctrl + clic para alternar. También accesible con teclado (botón con estado, no solo clic).
- **Barra de selección:** al haber fuentes seleccionadas aparece una barra con "N seleccionadas", **Compare** y **Clear**. Opción "Compare" también en el menú de clic derecho.
- **Vista de comparación:** una columna por fuente, con un único texto editable y un tamaño compartidos, selector de variante (peso y estilo) por columna, y un resumen de cobertura (español, símbolos) debajo de cada una.
- **Límite:** de 2 a 6 fuentes (para que quepan lado a lado); avisar al pasar el máximo.
- Salir con Escape o con un botón de cerrar, conservando la selección.

**Terminado cuando:** se pueden seleccionar 2 o más fuentes por el círculo y con Ctrl + clic, abrir la comparación, escribir un texto y ver todas con ese texto. Pruebas de la lógica de selección (alternar, límite, limpiar).

---

## ✅ 4. Poder cerrar el aviso "N files with errors"

**Pedido:** poner una **X** junto a "2 files with errors" en la barra superior de escaneo, para que quede solo "289 families from 532 files" y el botón Rescan.

**Propuesta:**
- Botón ✕ en el aviso de errores que lo oculta (y cierra la lista de errores si está abierta).
- El aviso vuelve a aparecer solo si un nuevo escaneo produce errores.
- Los errores **no se pierden**: siguen consultables desde Settings (enlace "View scan errors").

**Terminado cuando:** al pulsar ✕ la barra muestra solo el conteo de familias y archivos y Rescan; tras un nuevo escaneo con errores, el aviso reaparece.

---

## ✅ 5. Texto "Showing X fonts out of N"

**Pedido:** un texto que diga cuántas fuentes se muestran del total, teniendo en cuenta filtros y etiquetas.

**Hoy:** la fila "X of Y families" solo aparece con filtros activos y compara contra la sección actual, no contra toda la biblioteca.

**Propuesta:**
- Texto visible siempre: **"Showing X fonts out of N"**, donde N es el total de la biblioteca y X lo que queda tras aplicar a la vez la sección activa, la búsqueda por nombre, las etiquetas, los favoritos y los filtros.
- Mantener las "fichas" de filtros activos junto al texto, con su "Clear all".
- Sigue el aviso de fuentes sin analizar excluidas por filtros de análisis.
- Nota de terminología: aquí "fonts" significa **familias**. La barra de escaneo conserva "families from files" para no confundir familias con archivos.

**Terminado cuando:** con la biblioteca completa dice "Showing 289 fonts out of 289"; al combinar filtros, etiqueta y búsqueda, X coincide con las tarjetas visibles. Prueba automática del conteo.

---

## Orden sugerido

1. Punto 4 y punto 5 (cambios pequeños y de bajo riesgo, comparten la barra superior).
2. Punto 2 (diseño del panel derecho).
3. Punto 1 (requiere reproducir el caso y tocar Rust para leer el archivo de la variante).
4. Punto 3 (el más grande; reutiliza el preview exacto del punto 1).

## Después

- **Fase 6 — Calidad y distribución:** pruebas en Windows 10 y 11, archivos dañados y sin permisos, consumo de memoria y tiempo, instalador y documentación.
- Ideas ya mencionadas, sin prioridad: variaciones de ancho por separado (Semi Condensed, Extra Condensed, Roman), nombres literales "Ultra Bold" y "Extra Bold" como entradas distintas, filtros inteligentes guardables por el usuario.

---

## Hecho en la 5.3

- Slider de tamaño: ya no congela con miles de fuentes (variable CSS + `content-visibility`). Medido: ~1100 ms → ~32 ms con 1500 fuentes.
- Importar fuentes (.ttf/.otf/.ttc/.otc) para el usuario actual y exportar las mostradas o las seleccionadas.
- Pendiente de probar en Windows: el registro de la fuente y el aviso al sistema (código específico de Windows que no se pudo compilar en el entorno de desarrollo).
- Ideas futuras: arrastrar y soltar archivos para importar, desinstalar fuentes, instalar para todos los usuarios (requiere administrador), exportar a .zip.
