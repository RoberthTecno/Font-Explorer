import type { ScannedFamily } from "./scan";

/** Estado del análisis tipográfico de una familia (ver spec §6.5). */
export type AnalysisStatus = "full" | "partial" | "none" | "unanalyzed";

/** Familia tipográfica tal como la muestra la biblioteca. */
export interface FontFamilyItem {
  id: string;
  familyName: string;
  /** Nombre usado en CSS para renderizar el preview. */
  cssFamily: string;
  status: AnalysisStatus;
  isFavorite: boolean;
  variantCount: number;
  isVariable: boolean;
  /** Datos reales del escaneo; null en datos de ejemplo (= no analizado). */
  scanned: ScannedFamily | null;
}

export type Section =
  | { kind: "all" }
  | { kind: "favorites" }
  | { kind: "recents" }
  | { kind: "smart"; id: string }
  | { kind: "tag"; id: number }
  | { kind: "settings" };

export type ViewMode = "grid" | "list";

/** Filtros combinables (todas las condiciones activas deben cumplirse a la vez). */
export interface Filters {
  /** Pesos que la familia debe tener (todos los marcados). */
  weights: number[];
  italic: boolean;
  oblique: boolean;
  upper: boolean;
  lower: boolean;
  /** áéíóúÁÉÍÓÚ completas. */
  accents: boolean;
  /** Todo el conjunto español (tildes, ñ, ü, ¿ ¡). */
  spanish: boolean;
  /** Todos los grupos de símbolos especiales. */
  symbols: boolean;
  favorites: boolean;
  /** Etiquetas que la familia debe tener (todas las marcadas). */
  tagIds: number[];
}
