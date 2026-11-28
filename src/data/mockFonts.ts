// DATOS FICTICIOS (solo Fase 1). Son nombres de fuentes que suelen venir con Windows
// para que el preview se vea; el estado, los favoritos, las etiquetas y los conteos
// son inventados y NO representan análisis reales.
import type { FontFamilyItem, AnalysisStatus } from "../types/font";
import { SAMPLE_EN, SAMPLE_ES } from "./sampleTexts";

type Row = [string, AnalysisStatus, number, boolean, string[]?];

const rows: Row[] = [
  ["Arial", "full", 9, false, ["Sans Serif"]],
  ["Bahnschrift", "full", 15, true, ["Moderna"]],
  ["Calibri", "full", 6, false, ["Sans Serif"]],
  ["Cambria", "full", 4, false, ["Serif"]],
  ["Candara", "partial", 4, false],
  ["Comic Sans MS", "partial", 4, false, ["Display"]],
  ["Consolas", "full", 4, false],
  ["Constantia", "full", 4, true, ["Elegante", "Serif"]],
  ["Corbel", "partial", 4, false],
  ["Courier New", "partial", 4, false, ["Retro"]],
  ["Ebrima", "unanalyzed", 2, false],
  ["Franklin Gothic Medium", "unanalyzed", 2, false, ["Gótica"]],
  ["Gabriola", "partial", 1, true, ["Manuscrita", "Elegante"]],
  ["Georgia", "full", 4, false, ["Serif", "Elegante"]],
  ["Impact", "partial", 1, false, ["Display"]],
  ["Ink Free", "unanalyzed", 1, false, ["Manuscrita"]],
  ["Lucida Console", "partial", 1, false, ["Retro"]],
  ["Palatino Linotype", "full", 4, false, ["Serif", "Vintage"]],
  ["Segoe Script", "unanalyzed", 2, true, ["Manuscrita"]],
  ["Segoe UI", "full", 12, false, ["Moderna", "Sans Serif"]],
  ["Sitka Text", "unanalyzed", 8, false],
  ["Tahoma", "full", 2, false],
  ["Times New Roman", "full", 4, false, ["Serif", "Vintage"]],
  ["Trebuchet MS", "partial", 4, false],
  ["Verdana", "full", 4, false, ["Sans Serif"]],
  ["Old English Text MT", "unanalyzed", 1, false, ["Gótica", "Vintage"]],
  ["Rockwell", "partial", 4, false, ["Retro"]],
  ["Gill Sans MT", "full", 6, false, ["Minimalista"]],
];

export const mockFonts: FontFamilyItem[] = rows.map(([name, status, variants, fav], i) => ({
  id: `mock-${i + 1}`,
  familyName: name,
  cssFamily: name,
  status,
  isFavorite: fav,
  variantCount: variants,
  isVariable: name === "Bahnschrift" || name === "Segoe UI",
  scanned: null,
}));

/** Etiquetas de ejemplo por fuente (solo para la vista previa en navegador). */
export const mockFontTags: Record<string, string[]> = Object.fromEntries(
  rows.map(([, , , , tags], i) => [`mock-${i + 1}`, tags ?? []]),
);
/** Recientes de ejemplo, el más reciente primero. */
export const mockRecents = [3, 11, 19, 7, 23].map((i) => `mock-${i + 1}`);

export const mockTags = [
  "Moderna", "Gótica", "Vintage", "Minimalista", "Elegante", "Retro",
  "Manuscrita", "Serif", "Sans Serif", "Display",
];

export const previewChips: { label: string; text: string }[] = [
  { label: "123", text: "0123456789" },
  { label: "ABC", text: "ABCDEFGHIJKLMNOPQRSTUVWXYZ" },
  { label: "abc", text: "abcdefghijklmnopqrstuvwxyz" },
  { label: "fox", text: SAMPLE_EN },
  { label: "español", text: SAMPLE_ES },
  { label: "lorem", text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit" },
  { label: "symbols", text: "© ® ™ € £ ¥ § ¶ † ‡ • ← → ★ ♥" },
];
