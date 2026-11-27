export interface TransferItem {
  name: string;
  status: "done" | "skipped" | "error";
  message: string;
}

/** Espejo de TransferResult (src-tauri/src/commands/transfer.rs). */
export interface TransferResult {
  done: number;
  skipped: number;
  failed: number;
  items: TransferItem[];
  folder: string;
}

export interface TransferState {
  kind: "import" | "export";
  phase: "confirm" | "running" | "done" | "error";
  paths: string[];
  ids: string[];
  destination: string;
  progress: { processed: number; total: number } | null;
  result: TransferResult | null;
  error: string;
}
