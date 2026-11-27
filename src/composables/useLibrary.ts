import { computed, ref, watch } from "vue";
import { mockFontTags, mockFonts, mockRecents, mockTags } from "../data/mockFonts";
import * as api from "../services/fontService";
import { familyStatus, variationById, VARIATIONS } from "../services/analysis";
import { activeCount, analysisActive, emptyFilters, matchesFilters } from "../services/filters";
import type { Filters, FontFamilyItem, Section, ViewMode } from "../types/font";
import type { CharSetDef, ScanProgress, ScanResult, ScannedFamily } from "../types/scan";
import type { Tag, UserData } from "../types/userdata";
import type { TransferState } from "../types/transfer";

// Estado compartido a nivel de módulo. En Tauri la fuente de verdad es SQLite (vía comandos);
// en un navegador normal todo vive en memoria con datos de ejemplo.
const DEFAULT_SIZE = 34;

const fonts = ref<FontFamilyItem[]>(api.isTauri ? [] : mockFonts.map((f) => ({ ...f })));
const tags = ref<Tag[]>(api.isTauri ? [] : mockTags.map((name, i) => ({ id: i + 1, name, color: null })));
let localTagSeq = tags.value.length;
const fontTagMap = ref<Record<string, number[]>>(
  api.isTauri ? {} : Object.fromEntries(Object.entries(mockFontTags).map(([id, names]) => [
    id, names.map((n) => tags.value.find((t) => t.name === n)!.id),
  ])),
);
const favoriteIds = ref(new Set<string>(api.isTauri ? [] : mockFonts.filter((f) => f.isFavorite).map((f) => f.id)));
const recents = ref<string[]>(api.isTauri ? [] : [...mockRecents]);

const section = ref<Section>({ kind: "all" });
const search = ref("");
const viewMode = ref<ViewMode>("grid");
const previewSize = ref(DEFAULT_SIZE);
const previewText = ref("");
const sidebarOpen = ref(true);
const selectedId = ref<string | null>(null);
const filters = ref<Filters>(emptyFilters());
const transfer = ref<TransferState | null>(null);
let pendingRescan = false;
const MAX_COMPARE = 6;
const compareIds = ref<string[]>([]);
const compareOpen = ref(false);
const compareNotice = ref("");
const contextMenu = ref<{ fontId: string; x: number; y: number } | null>(null);
const scanning = ref(false);
const scanProgress = ref<ScanProgress | null>(null);
const lastScan = ref<ScanResult | null>(null);

const selectedFont = computed(() => fonts.value.find((f) => f.id === selectedId.value) ?? null);
const charSets = computed<CharSetDef[]>(() => lastScan.value?.charSets ?? []);
const sortedTags = computed(() => [...tags.value].sort((a, b) => a.name.localeCompare(b.name)));
const tagsOf = (fontId: string) =>
  (fontTagMap.value[fontId] ?? []).map((id) => tags.value.find((t) => t.id === id)).filter((t): t is Tag => !!t);

const norm = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

const sectionFiltered = computed(() => {
  const s = section.value;
  switch (s.kind) {
    case "favorites":
      return fonts.value.filter((f) => f.isFavorite);
    case "recents": {
      const order = recents.value;
      return order.map((id) => fonts.value.find((f) => f.id === id)).filter((f): f is FontFamilyItem => !!f);
    }
    case "smart": {
      const v = variationById(s.id);
      return v ? fonts.value.filter((f) => f.scanned && v.test(f.scanned)) : [];
    }
    case "tag":
      return fonts.value.filter((f) => fontTagMap.value[f.id]?.includes(s.id));
    default:
      return fonts.value;
  }
});

/** Familias de la sección actual que coinciden con la búsqueda por nombre. */
const baseFonts = computed(() => {
  const q = norm(search.value.trim());
  return q ? sectionFiltered.value.filter((f) => norm(f.familyName).includes(q)) : sectionFiltered.value;
});

/** Cuántas familias tienen cada variación; solo hay datos para fuentes analizadas. */
const variationCounts = computed<Record<string, number>>(() => {
  const counts: Record<string, number> = {};
  for (const v of VARIATIONS) counts[v.id] = 0;
  for (const f of fonts.value) {
    if (!f.scanned) continue;
    for (const v of VARIATIONS) if (v.test(f.scanned)) counts[v.id]++;
  }
  return counts;
});
const hasAnalysis = computed(() => fonts.value.some((f) => f.scanned));

const activeFilterCount = computed(() => activeCount(filters.value));

/** Sección + búsqueda + filtros, todo combinado con "y". */
const visibleFonts = computed(() =>
  activeFilterCount.value === 0
    ? baseFonts.value
    : baseFonts.value.filter((f) => matchesFilters(f, filters.value, charSets.value, fontTagMap.value[f.id] ?? [])),
);

/** Cuántas familias quedan fuera de los filtros de análisis solo por no estar analizadas. */
const unanalyzedExcluded = computed(() =>
  analysisActive(filters.value) ? baseFonts.value.filter((f) => !f.scanned).length : 0,
);

const clearFilters = () => { filters.value = emptyFilters(); };

// ---------- Selección para comparar ----------
let noticeTimer: ReturnType<typeof setTimeout> | undefined;
function toggleCompare(id: string): boolean {
  if (compareIds.value.includes(id)) {
    compareIds.value = compareIds.value.filter((x) => x !== id);
    return true;
  }
  if (compareIds.value.length >= MAX_COMPARE) {
    compareNotice.value = `You can compare up to ${MAX_COMPARE} fonts at once`;
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => { compareNotice.value = ""; }, 2500);
    return false;
  }
  compareIds.value = [...compareIds.value, id];
  return true;
}
// ---------- Importar y exportar ----------
const msg = (e: unknown) => (typeof e === "string" ? e : (e as Error).message);

/** Familias a exportar: las que se muestran ahora (con filtros, etiquetas y búsqueda) o las seleccionadas con el círculo. */
function exportTargets(scope: "shown" | "selected"): string[] {
  const ids = scope === "shown" ? visibleFonts.value.map((f) => f.id) : compareIds.value;
  return ids.filter((id) => fonts.value.find((f) => f.id === id)?.scanned);
}

async function startImport() {
  if (!api.isTauri || transfer.value) return;
  const paths = await api.pickFontFiles();
  if (!paths.length) return;
  transfer.value = { kind: "import", phase: "confirm", paths, ids: [], destination: "", progress: null, result: null, error: "" };
}

async function confirmImport() {
  const t = transfer.value;
  if (!t || t.kind !== "import" || t.phase !== "confirm") return;
  t.phase = "running";
  try {
    t.result = await api.installFonts(t.paths);
    t.phase = "done";
    if (t.result.done > 0) {
      if (scanning.value) pendingRescan = true; else await rescan();
    }
  } catch (e) { t.error = msg(e); t.phase = "error"; }
}

async function startExport(scope: "shown" | "selected") {
  if (!api.isTauri || transfer.value) return;
  const ids = exportTargets(scope);
  if (!ids.length) return;
  const folder = await api.pickFolder();
  if (!folder) return;
  const t: TransferState = { kind: "export", phase: "running", paths: [], ids, destination: folder, progress: null, result: null, error: "" };
  transfer.value = t;
  try {
    transfer.value.result = await api.exportFonts(ids, folder);
    transfer.value.phase = "done";
  } catch (e) { if (transfer.value) { transfer.value.error = msg(e); transfer.value.phase = "error"; } }
}

function closeTransfer() { if (transfer.value?.phase !== "running") transfer.value = null; }

function clearCompare() { compareIds.value = []; compareOpen.value = false; }
function openCompare() { if (compareIds.value.length >= 2) compareOpen.value = true; }
function closeCompare() { compareOpen.value = false; }

function openContextMenu(fontId: string, x: number, y: number) { contextMenu.value = { fontId, x, y }; }
function closeContextMenu() { contextMenu.value = null; }

function sectionTitle(s: Section): string {
  switch (s.kind) {
    case "favorites": return "Favorites";
    case "recents": return "Recents";
    case "smart": return variationById(s.id)?.label ?? "Smart Filter";
    case "tag": return `# ${tags.value.find((t) => t.id === s.id)?.name ?? "Tag"}`;
    case "settings": return "Settings";
    default: return "All Fonts";
  }
}

// ---------- Persistencia de preferencias ----------
let prefsReady = false;
let prefsTimer: ReturnType<typeof setTimeout> | undefined;
watch([previewSize, previewText, viewMode], () => {
  if (!api.isTauri || !prefsReady) return;
  clearTimeout(prefsTimer);
  prefsTimer = setTimeout(() => {
    const save = (k: string, v: string) => api.setSetting(k, v).catch(() => {});
    save("preview_size", String(previewSize.value));
    save("preview_text", previewText.value);
    save("view_mode", viewMode.value);
  }, 300);
});

function resetPreferences() {
  previewSize.value = DEFAULT_SIZE;
  previewText.value = "";
  viewMode.value = "grid";
}

// ---------- Carga de datos ----------
function toItem(f: ScannedFamily, sets: CharSetDef[]): FontFamilyItem {
  return {
    id: f.id, familyName: f.familyName, cssFamily: f.familyName, status: familyStatus(f, sets),
    isFavorite: favoriteIds.value.has(f.id), variantCount: f.variantCount, isVariable: f.isVariable, scanned: f,
  };
}

function applyUserData(ud: UserData) {
  tags.value = ud.tags;
  const map: Record<string, number[]> = {};
  for (const ft of ud.fontTags) (map[ft.fontId] ??= []).push(ft.tagId);
  fontTagMap.value = map;
  favoriteIds.value = new Set(ud.favorites);
  recents.value = ud.recents;
  const size = Number(ud.settings.preview_size);
  if (size >= 16 && size <= 72) previewSize.value = size;
  if (ud.settings.preview_text !== undefined) previewText.value = ud.settings.preview_text;
  if (ud.settings.view_mode === "grid" || ud.settings.view_mode === "list") viewMode.value = ud.settings.view_mode;
}

function applyScan(r: ScanResult) {
  fonts.value = r.families.map((f) => toItem(f, r.charSets));
  lastScan.value = r;
  scanning.value = false;
  scanProgress.value = null;
  if (pendingRescan) { pendingRescan = false; void rescan(); }
}

let wired = false;
async function initScan() {
  if (!api.isTauri || wired) return;
  wired = true;
  try { applyUserData(await api.getUserData()); } catch { /* se inicia con valores por defecto */ }
  prefsReady = true;
  await api.onScanProgress((p) => { scanProgress.value = p; });
  await api.onScanDone(applyScan);
  await api.onTransferProgress((p) => { if (transfer.value) transfer.value.progress = p; });
  const last = await api.getLastScan();
  if (last) applyScan(last); // la biblioteca guardada aparece al instante…
  await rescan(); // …y se refresca en segundo plano (los archivos sin cambios no se releen)
}

async function rescan() {
  if (!api.isTauri || scanning.value) return;
  scanning.value = true;
  scanProgress.value = null;
  try { await api.startScan(); } catch { scanning.value = false; }
}

function cancelScan() { return api.cancelScan(); }

// ---------- Favoritos y recientes ----------
function toggleFavorite(id: string) {
  const f = fonts.value.find((x) => x.id === id);
  if (!f) return;
  const next = !f.isFavorite;
  const apply = (v: boolean) => {
    f.isFavorite = v;
    const set = new Set(favoriteIds.value);
    if (v) { set.add(id); } else { set.delete(id); }
    favoriteIds.value = set;
  };
  apply(next);
  if (api.isTauri) api.setFavorite(id, next).catch(() => apply(!next));
}

function openFont(id: string) {
  selectedId.value = id;
  recents.value = [id, ...recents.value.filter((x) => x !== id)].slice(0, 50);
  if (api.isTauri) api.markOpened(id).catch(() => {});
}

// ---------- Etiquetas ----------
async function call<T>(p: Promise<T>): Promise<T> {
  try { return await p; } catch (e) { throw new Error(typeof e === "string" ? e : (e as Error).message); }
}
function cleanName(name: string): string {
  const n = name.trim();
  if (!n) throw new Error("Tag name can't be empty");
  return n;
}
const nameTaken = (n: string, exceptId?: number) =>
  tags.value.some((t) => t.id !== exceptId && t.name.toLowerCase() === n.toLowerCase());

async function createTag(name: string): Promise<Tag> {
  const n = cleanName(name);
  let tag: Tag;
  if (api.isTauri) {
    tag = await call(api.createTag(n, null));
  } else {
    if (nameTaken(n)) throw new Error("A tag with that name already exists");
    tag = { id: ++localTagSeq, name: n, color: null };
  }
  tags.value = [...tags.value, tag];
  return tag;
}

async function renameTag(id: number, name: string) {
  const n = cleanName(name);
  if (api.isTauri) await call(api.renameTag(id, n));
  else if (nameTaken(n, id)) throw new Error("A tag with that name already exists");
  tags.value = tags.value.map((t) => (t.id === id ? { ...t, name: n } : t));
}

async function deleteTag(id: number) {
  if (api.isTauri) await call(api.deleteTag(id));
  tags.value = tags.value.filter((t) => t.id !== id);
  const map: Record<string, number[]> = {};
  for (const [fid, ids] of Object.entries(fontTagMap.value)) map[fid] = ids.filter((x) => x !== id);
  fontTagMap.value = map;
  if (section.value.kind === "tag" && section.value.id === id) section.value = { kind: "all" };
}

async function addTagToFont(fontId: string, tagId: number) {
  if (api.isTauri) await call(api.addFontTag(fontId, tagId));
  const cur = fontTagMap.value[fontId] ?? [];
  if (!cur.includes(tagId)) fontTagMap.value = { ...fontTagMap.value, [fontId]: [...cur, tagId] };
}

async function removeTagFromFont(fontId: string, tagId: number) {
  if (api.isTauri) await call(api.removeFontTag(fontId, tagId));
  fontTagMap.value = { ...fontTagMap.value, [fontId]: (fontTagMap.value[fontId] ?? []).filter((x) => x !== tagId) };
}

export function useLibrary() {
  return {
    fonts, section, search, viewMode, previewSize, previewText, sidebarOpen,
    visibleFonts, toggleFavorite, sectionTitle, selectedId, selectedFont, charSets, openFont,
    scanning, scanProgress, lastScan, initScan, rescan, cancelScan,
    tags, sortedTags, tagsOf, createTag, renameTag, deleteTag, addTagToFont, removeTagFromFont,
    variationCounts, hasAnalysis, resetPreferences, filters, activeFilterCount, baseFonts, unanalyzedExcluded, clearFilters,
    contextMenu, openContextMenu, closeContextMenu,
    totalFonts: computed(() => fonts.value.length),
    transfer, exportTargets, startImport, confirmImport, startExport, closeTransfer,
    compareIds, compareOpen, compareNotice, MAX_COMPARE, toggleCompare, clearCompare, openCompare, closeCompare,
  };
}
