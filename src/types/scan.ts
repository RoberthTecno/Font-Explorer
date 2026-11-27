// Espejo de los tipos serializados por src-tauri/src/fonts/types.rs (camelCase).
export type Source = "system" | "user";
export type ScanErrorKind = "missing" | "unreadable" | "invalid";

export interface AxisInfo {
  tag: string;
  minValue: number;
  defaultValue: number;
  maxValue: number;
}

export type CharCategory = "latin" | "spanish" | "symbols";

export interface CharSetDef {
  id: string;
  label: string;
  category: CharCategory;
  chars: string;
}

export interface FaceInfo {
  faceIndex: number;
  subfamilyName: string;
  fullName: string | null;
  weightClass: number;
  /** Ancho OS/2: 1 ultra condensada … 5 normal … 9 ultra expandida. */
  widthClass: number;
  isItalic: boolean;
  isOblique: boolean;
  isVariable: boolean;
  axes: AxisInfo[];
  glyphCount: number;
  /** Caracteres de los conjuntos que esta cara NO tiene. */
  missing: string;
}

export interface FamilyFile {
  path: string;
  fileName: string;
  format: string;
  size: number;
  modifiedAt: number | null;
  source: Source;
  faces: FaceInfo[];
}

export interface ScannedFamily {
  id: string;
  familyName: string;
  isVariable: boolean;
  variantCount: number;
  files: FamilyFile[];
}

export interface ScanError {
  path: string;
  fileName: string;
  kind: ScanErrorKind;
  message: string;
}

export interface ScanResult {
  charSets: CharSetDef[];
  families: ScannedFamily[];
  errors: ScanError[];
  filesTotal: number;
  filesOk: number;
  filesSkipped: number;
  /** Archivos cuyo resultado se reutilizó de la base de datos. */
  filesReused: number;
  cancelled: boolean;
  scannedAt: number;
  durationMs: number;
}

export interface ScanProgress {
  processed: number;
  total: number;
  currentFile: string;
  errors: number;
}
