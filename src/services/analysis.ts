// Interpretación de los datos del escaneo. Funciones puras (probadas en analysis.test.ts).
import type { AnalysisStatus } from "../types/font";
import type { CharSetDef, FaceInfo, FamilyFile, ScannedFamily } from "../types/scan";

export const WEIGHT_NAMES: Record<number, string> = {
  100: "Thin", 200: "ExtraLight", 300: "Light", 400: "Regular", 500: "Medium",
  600: "SemiBold", 700: "Bold", 800: "ExtraBold", 900: "Black",
};

export function weightLabel(w: number): string {
  if (WEIGHT_NAMES[w]) return WEIGHT_NAMES[w];
  const near = Math.round(w / 100) * 100;
  return WEIGHT_NAMES[near] ? `${WEIGHT_NAMES[near]} (${w})` : String(w);
}

export interface SetCoverage {
  set: CharSetDef;
  present: string[];
  missing: string[];
}

export function faceCoverage(face: FaceInfo, sets: CharSetDef[]): SetCoverage[] {
  const missing = new Set([...face.missing]);
  return sets.map((set) => {
    const chars = [...set.chars];
    return { set, present: chars.filter((c) => !missing.has(c)), missing: chars.filter((c) => missing.has(c)) };
  });
}

/** Cumple = todo A–Z, a–z y el conjunto español. Parcial = tiene algo pero no todo. */
export function faceStatus(face: FaceInfo, sets: CharSetDef[]): AnalysisStatus {
  const core = faceCoverage(face, sets).filter((c) => c.set.category !== "symbols");
  if (core.length === 0) return "unanalyzed";
  if (core.every((c) => c.missing.length === 0)) return "full";
  return core.some((c) => c.present.length > 0) ? "partial" : "none";
}

const RANK: Record<AnalysisStatus, number> = { unanalyzed: -1, none: 0, partial: 1, full: 2 };

/** Regla de la v1: basta con que AL MENOS UNA cara de la familia cumpla (spec §8). */
export function familyStatus(f: ScannedFamily, sets: CharSetDef[]): AnalysisStatus {
  let best: AnalysisStatus = "unanalyzed";
  for (const file of f.files)
    for (const face of file.faces) {
      const s = faceStatus(face, sets);
      if (RANK[s] > RANK[best]) best = s;
    }
  return best;
}

const allFaces = (f: ScannedFamily) => f.files.flatMap((x) => x.faces);

/** Pesos disponibles. En fuentes variables se expanden por el rango del eje `wght`. */
export function familyWeights(f: ScannedFamily): number[] {
  const out = new Set<number>();
  for (const face of allFaces(f)) {
    out.add(face.weightClass);
    const w = face.axes.find((a) => a.tag === "wght");
    if (w) for (let n = 100; n <= 900; n += 100) if (n >= w.minValue && n <= w.maxValue) out.add(n);
  }
  return [...out].sort((a, b) => a - b);
}

export function familyStyles(f: ScannedFamily): { italic: boolean; oblique: boolean } {
  const faces = allFaces(f);
  return {
    italic: faces.some((x) => x.isItalic || x.axes.some((a) => a.tag === "ital" && a.maxValue > 0)),
    oblique: faces.some((x) => x.isOblique || x.axes.some((a) => a.tag === "slnt" && (a.minValue < 0 || a.maxValue > 0))),
  };
}

export function hasBoldAndItalic(f: ScannedFamily): boolean {
  return familyWeights(f).some((w) => w >= 700) && familyStyles(f).italic;
}

// ---------- Variaciones (filtros inteligentes de la barra lateral) ----------
export type VariationGroup = "Weight" | "Style" | "Width";

export interface Variation {
  id: string;
  label: string;
  group: VariationGroup;
  /** Una familia coincide si AL MENOS UNA de sus variantes tiene esta variación. */
  test: (f: ScannedFamily) => boolean;
}

const hasWeight = (w: number) => (f: ScannedFamily) => familyWeights(f).includes(w);

const CONDENSED_NAME = /narrow|condensed/i;
const EXPANDED_NAME = /expanded|extended|wide/i;

/** Condensada/estrecha: clase de ancho 1–4, eje `wdth` por debajo de 100 o nombre de estilo propio. */
export function isCondensed(f: ScannedFamily): boolean {
  return allFaces(f).some((x) =>
    x.widthClass <= 4 || CONDENSED_NAME.test(x.subfamilyName) || x.axes.some((a) => a.tag === "wdth" && a.minValue < 100));
}
export function isExpanded(f: ScannedFamily): boolean {
  return allFaces(f).some((x) =>
    x.widthClass >= 6 || EXPANDED_NAME.test(x.subfamilyName) || x.axes.some((a) => a.tag === "wdth" && a.maxValue > 100));
}

export const VARIATIONS: Variation[] = [
  { id: "w100", label: "Thin", group: "Weight", test: hasWeight(100) },
  { id: "w200", label: "Extra / Ultra Light", group: "Weight", test: hasWeight(200) },
  { id: "w300", label: "Light", group: "Weight", test: hasWeight(300) },
  { id: "w400", label: "Regular", group: "Weight", test: hasWeight(400) },
  { id: "w500", label: "Medium", group: "Weight", test: hasWeight(500) },
  { id: "w600", label: "Semi / Demi Bold", group: "Weight", test: hasWeight(600) },
  { id: "w700", label: "Bold", group: "Weight", test: hasWeight(700) },
  { id: "w800", label: "Extra / Ultra Bold", group: "Weight", test: hasWeight(800) },
  { id: "w900", label: "Black / Heavy", group: "Weight", test: hasWeight(900) },
  { id: "italic", label: "Italic", group: "Style", test: (f) => familyStyles(f).italic },
  { id: "oblique", label: "Oblique", group: "Style", test: (f) => familyStyles(f).oblique },
  { id: "condensed", label: "Condensed / Narrow", group: "Width", test: isCondensed },
  { id: "expanded", label: "Expanded / Wide", group: "Width", test: isExpanded },
];

export const variationById = (id: string) => VARIATIONS.find((v) => v.id === id);

// ---------- Variante representativa (preview de las tarjetas) ----------

/** Regular si existe; si no, la variante vertical y de ancho normal más cercana a peso 400. */
export function representativeFace(f: ScannedFamily): { file: FamilyFile; face: FaceInfo } | null {
  let best: { file: FamilyFile; face: FaceInfo } | null = null;
  let bestScore = Infinity;
  for (const file of f.files)
    for (const face of file.faces) {
      const score = Math.abs(face.weightClass - 400) + (face.isItalic || face.isOblique ? 1000 : 0) + (face.widthClass !== 5 ? 500 : 0);
      if (score < bestScore) { best = { file, face }; bestScore = score; }
    }
  return best;
}
