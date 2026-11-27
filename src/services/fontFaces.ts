// Dibuja cada variante con SU archivo real. El nombre de familia del sistema no distingue bien
// variantes como Ultra Bold (Windows las agrupa distinto), así que se carga el archivo como FontFace.
import { invoke } from "@tauri-apps/api/core";
import { reactive } from "vue";
import { isTauri } from "./fontService";
import type { AxisInfo, FaceInfo, FamilyFile } from "../types/scan";

export interface FaceDesc {
  path: string;
  faceIndex: number;
  format: string;
  size: number;
  weightClass: number;
  isItalic: boolean;
  isOblique: boolean;
  axes: AxisInfo[];
}

/** En tarjetas no se cargan archivos grandes (p. ej. CJK); se usa el nombre de familia como respaldo. */
export const CARD_MAX_BYTES = 12 * 1024 * 1024;
export const DETAIL_MAX_BYTES = 64 * 1024 * 1024;
const MAX_LOADED = 300;
const MAX_PARALLEL = 4;

const loaded = reactive(new Map<string, string>()); // clave de cara → nombre CSS registrado
const failed = new Set<string>();
const pending = new Map<string, Promise<void>>();
const registered = new Map<string, FontFace>();
let seq = 0;
let active = 0;
const waiting: (() => void)[] = [];

export const keyOf = (d: FaceDesc) => `${d.path}#${d.faceIndex}`;
export const descOf = (file: FamilyFile, face: FaceInfo): FaceDesc => ({
  path: file.path, faceIndex: face.faceIndex, format: file.format, size: file.size,
  weightClass: face.weightClass, isItalic: face.isItalic, isOblique: face.isOblique, axes: face.axes,
});

const styleOf = (d: FaceDesc) => (d.isItalic ? "italic" : d.isOblique ? "oblique" : "normal");
const clampWeight = (w: number) => Math.min(1000, Math.max(1, Math.round(w) || 400));
const cssName = (n: string) => n.replace(/['\\]/g, "\\$&");

/** Estilo CSS de una variante: usa el archivo real si ya cargó; si no, familia + peso + estilo. */
export function previewStyle(family: string, d: FaceDesc | null, px?: number): Record<string, string | number> {
  const real = d ? loaded.get(keyOf(d)) : undefined;
  return {
    fontFamily: real ? `'${real}', '${cssName(family)}', sans-serif` : `'${cssName(family)}', sans-serif`,
    fontWeight: d ? clampWeight(d.weightClass) : 400,
    fontStyle: d ? styleOf(d) : "normal",
    ...(px ? { fontSize: `${px}px` } : {}),
  };
}

async function slot<T>(fn: () => Promise<T>): Promise<T> {
  if (active >= MAX_PARALLEL) await new Promise<void>((r) => waiting.push(r));
  active++;
  try { return await fn(); } finally { active--; waiting.shift()?.(); }
}

function evict() {
  while (loaded.size > MAX_LOADED) {
    const oldest = loaded.keys().next().value as string;
    const f = registered.get(oldest);
    if (f) document.fonts.delete(f);
    registered.delete(oldest);
    loaded.delete(oldest);
  }
}

/** Carga el archivo de la variante (si hace falta). Nunca lanza: ante cualquier fallo queda el respaldo. */
export function ensureFace(d: FaceDesc, maxBytes: number): Promise<void> {
  const key = keyOf(d);
  if (!isTauri || typeof FontFace === "undefined") return Promise.resolve();
  if (loaded.has(key) || failed.has(key) || d.size > maxBytes) return Promise.resolve();
  // Un FontFace no puede elegir una cara dentro de un .ttc/.otc: solo se carga la primera.
  if ((d.format === "TTC" || d.format === "OTC") && d.faceIndex > 0) return Promise.resolve();
  const existing = pending.get(key);
  if (existing) return existing;

  const job = slot(async () => {
    try {
      const bytes = await invoke<ArrayBuffer>("read_font_file", { path: d.path });
      const name = `fx-${++seq}`;
      const wght = d.axes.find((a) => a.tag === "wght");
      const weight = wght ? `${clampWeight(wght.minValue)} ${clampWeight(wght.maxValue)}` : String(clampWeight(d.weightClass));
      const face = new FontFace(name, bytes, { weight, style: styleOf(d) });
      await face.load();
      document.fonts.add(face);
      registered.set(key, face);
      loaded.set(key, name);
      evict();
    } catch {
      failed.add(key);
    }
  }).finally(() => pending.delete(key));
  pending.set(key, job);
  return job;
}
