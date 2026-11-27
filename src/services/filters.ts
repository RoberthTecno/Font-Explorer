// Lógica pura de filtrado. Regla de la v1 (spec §8): una familia cumple un filtro si AL MENOS UNA
// de sus variantes lo cumple; todos los filtros activos se combinan con "y".
import type { Filters, FontFamilyItem } from "../types/font";
import type { CharSetDef, FaceInfo, ScannedFamily } from "../types/scan";
import { familyStyles, familyWeights } from "./analysis";

export const emptyFilters = (): Filters => ({
  weights: [], italic: false, oblique: false, upper: false, lower: false,
  accents: false, spanish: false, symbols: false, favorites: false, tagIds: [],
});

/** Filtros que requieren datos del análisis tipográfico. */
export const analysisActive = (f: Filters) =>
  f.weights.length > 0 || f.italic || f.oblique || f.upper || f.lower || f.accents || f.spanish || f.symbols;

export function activeCount(f: Filters): number {
  return f.weights.length + f.tagIds.length +
    [f.italic, f.oblique, f.upper, f.lower, f.accents, f.spanish, f.symbols, f.favorites].filter(Boolean).length;
}

/** Condiciones de cobertura activas: cada una indica qué conjuntos de caracteres debe tener completos. */
function coverageChecks(f: Filters): ((s: CharSetDef) => boolean)[] {
  const checks: ((s: CharSetDef) => boolean)[] = [];
  if (f.upper) checks.push((s) => s.id === "upper");
  if (f.lower) checks.push((s) => s.id === "lower");
  if (f.accents) checks.push((s) => s.id === "accents");
  if (f.spanish) checks.push((s) => s.category === "spanish");
  if (f.symbols) checks.push((s) => s.category === "symbols");
  return checks;
}

function faceCovers(face: FaceInfo, sets: CharSetDef[], pick: (s: CharSetDef) => boolean): boolean {
  const chosen = sets.filter(pick);
  return chosen.length > 0 && chosen.every((s) => [...s.chars].every((c) => !face.missing.includes(c)));
}

const allFaces = (fam: ScannedFamily) => fam.files.flatMap((x) => x.faces);

/** ¿Esta cara cumple TODAS las condiciones de cobertura activas? (para señalar la variante en el detalle). */
export function faceMatchesCoverage(face: FaceInfo, f: Filters, sets: CharSetDef[]): boolean {
  const checks = coverageChecks(f);
  return checks.length > 0 && checks.every((c) => faceCovers(face, sets, c));
}

export const hasCoverageFilters = (f: Filters) => coverageChecks(f).length > 0;

export function matchesFilters(item: FontFamilyItem, f: Filters, sets: CharSetDef[], tagIds: number[]): boolean {
  if (f.favorites && !item.isFavorite) return false;
  if (!f.tagIds.every((id) => tagIds.includes(id))) return false;
  if (!analysisActive(f)) return true;

  // Sin datos de análisis no se puede afirmar que cumpla: se excluye (no se presenta como "incompatible").
  const fam = item.scanned;
  if (!fam) return false;
  if (f.weights.length) {
    const have = familyWeights(fam);
    if (!f.weights.every((w) => have.includes(w))) return false;
  }
  const st = familyStyles(fam);
  if (f.italic && !st.italic) return false;
  if (f.oblique && !st.oblique) return false;
  const faces = allFaces(fam);
  return coverageChecks(f).every((check) => faces.some((face) => faceCovers(face, sets, check)));
}

export const resultsLabel = (shown: number, total: number) =>
  `Showing ${shown} ${shown === 1 ? "font" : "fonts"} out of ${total}`;
