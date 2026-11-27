<script setup lang="ts">
import { computed, ref } from "vue";
import { isTauri } from "../services/fontService";
import { useLibrary } from "../composables/useLibrary";

const { scanning, scanProgress, lastScan, rescan, cancelScan } = useLibrary();
const showErrors = ref(false);
// El aviso se puede cerrar; reaparece solo si un escaneo nuevo produce errores.
const dismissedAt = ref<number | null>(null);
const errorsVisible = computed(
  () => !!lastScan.value && lastScan.value.errors.length > 0 && dismissedAt.value !== lastScan.value.scannedAt,
);
function dismissErrors() {
  dismissedAt.value = lastScan.value?.scannedAt ?? null;
  showErrors.value = false;
}
const pct = computed(() => {
  const p = scanProgress.value;
  return p && p.total > 0 ? Math.round((p.processed / p.total) * 100) : 0;
});
const kindLabel = { missing: "Missing", unreadable: "Unreadable", invalid: "Invalid font" } as const;
</script>

<template>
  <div v-if="!isTauri" class="scan scan--note">
    Browser preview: showing sample data. Run the desktop app to scan your installed fonts.
  </div>
  <div v-else-if="scanning" class="scan" role="status">
    <span>Scanning fonts… {{ scanProgress?.processed ?? 0 }} of {{ scanProgress?.total ?? "…" }} files</span>
    <progress :value="pct" max="100" aria-label="Scan progress" />
    <span v-if="scanProgress?.errors" class="scan__err">{{ scanProgress.errors }} errors</span>
    <button class="btn" @click="cancelScan">Cancel</button>
  </div>
  <div v-else-if="lastScan" class="scan">
    <span>
      {{ lastScan.families.length }} families from {{ lastScan.filesOk }} files
      <template v-if="lastScan.cancelled"> (scan cancelled, partial results)</template>
    </span>
    <span v-if="errorsVisible" class="errchip">
      <button class="btn btn--link" :aria-expanded="showErrors" @click="showErrors = !showErrors">
        {{ lastScan.errors.length }} files with errors
      </button>
      <button class="errchip__x" aria-label="Dismiss the errors notice" title="Dismiss (see Settings for details)" @click="dismissErrors">✕</button>
    </span>
    <span class="scan__spacer" />
    <button class="btn" @click="rescan">Rescan</button>
  </div>
  <ul v-if="isTauri && showErrors && errorsVisible && lastScan && !scanning" class="scan__list">
    <li v-for="e in lastScan.errors" :key="e.path">
      <strong>{{ e.fileName }}</strong> — {{ kindLabel[e.kind] }}: {{ e.message }}
      <span class="scan__path">{{ e.path }}</span>
    </li>
  </ul>
</template>
