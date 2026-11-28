// Distribución de la vista de comparación.
//  · 1 a 3 fuentes → columnas lado a lado.
//  · 4 fuentes     → cuadros de 2 × 2.
//  · 5 o 6 fuentes → cuadros de 3 × 2.
export interface CompareLayout {
  kind: "columns" | "tiles";
  cols: number;
  rows: number;
}

export function compareLayout(count: number): CompareLayout {
  const n = Math.max(1, count);
  if (n <= 3) return { kind: "columns", cols: n, rows: 1 };
  const cols = n === 4 ? 2 : 3;
  return { kind: "tiles", cols, rows: Math.ceil(n / cols) };
}
