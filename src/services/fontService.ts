import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ScanProgress, ScanResult } from "../types/scan";

/** false cuando la UI se abre en un navegador normal (p. ej. `npm run dev` sin Tauri). */
export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const startScan = () => invoke<void>("start_scan");
export const cancelScan = () => invoke<void>("cancel_scan");
export const getLastScan = () => invoke<ScanResult | null>("get_last_scan");
export const isScanning = () => invoke<boolean>("is_scanning");

export const onScanProgress = (cb: (p: ScanProgress) => void): Promise<UnlistenFn> =>
  listen<ScanProgress>("scan-progress", (e) => cb(e.payload));
export const onScanDone = (cb: (r: ScanResult) => void): Promise<UnlistenFn> =>
  listen<ScanResult>("scan-done", (e) => cb(e.payload));

// ---- Datos del usuario (SQLite) ----
import type { Tag, UserData } from "../types/userdata";

export const getUserData = () => invoke<UserData>("get_user_data");
export const setFavorite = (fontId: string, value: boolean) => invoke<void>("set_favorite", { fontId, value });
export const markOpened = (fontId: string) => invoke<void>("mark_opened", { fontId });
export const createTag = (name: string, color: string | null) => invoke<Tag>("create_tag", { name, color });
export const renameTag = (id: number, name: string) => invoke<void>("rename_tag", { id, name });
export const deleteTag = (id: number) => invoke<void>("delete_tag", { id });
export const addFontTag = (fontId: string, tagId: number) => invoke<void>("add_font_tag", { fontId, tagId });
export const removeFontTag = (fontId: string, tagId: number) => invoke<void>("remove_font_tag", { fontId, tagId });
export const setSetting = (key: string, value: string) => invoke<void>("set_setting", { key, value });

// ---- Importar y exportar ----
import type { TransferResult } from "../types/transfer";

export const installFonts = (paths: string[]) => invoke<TransferResult>("install_fonts", { paths });
export const exportFonts = (fontIds: string[], destination: string) =>
  invoke<TransferResult>("export_fonts", { fontIds, destination });
export const onTransferProgress = (cb: (p: { processed: number; total: number }) => void): Promise<UnlistenFn> =>
  listen<{ processed: number; total: number }>("transfer-progress", (e) => cb(e.payload));

/** Selector nativo de archivos de fuentes (varios a la vez). Devuelve [] si se cancela. */
export async function pickFontFiles(): Promise<string[]> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({
    multiple: true, directory: false, title: "Choose fonts to install",
    filters: [{ name: "Fonts", extensions: ["ttf", "otf", "ttc", "otc"] }],
  });
  return Array.isArray(r) ? r : r ? [r] : [];
}

/** Selector nativo de carpeta. Devuelve null si se cancela. */
export async function pickFolder(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({ multiple: false, directory: true, title: "Choose a folder to export to" });
  return typeof r === "string" ? r : null;
}
