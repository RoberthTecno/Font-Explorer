<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import SampleChips from "./SampleChips.vue";
import { SAMPLE_EN } from "../data/sampleTexts";
import { useLibrary } from "../composables/useLibrary";
import { compareLayout } from "../services/compareLayout";
import { faceCoverage, faceStatus, familyWeights, representativeFace, weightLabel } from "../services/analysis";
import { DETAIL_MAX_BYTES, descOf, ensureFace, previewStyle } from "../services/fontFaces";
import type { FontFamilyItem } from "../types/font";
import type { FaceInfo, FamilyFile } from "../types/scan";

const { fonts, compareIds, compareOpen, closeCompare, toggleCompare, charSets, previewText } = useLibrary();

interface FaceRow { key: string; file: FamilyFile; face: FaceInfo }
const DEFAULT_TEXT = SAMPLE_EN;
const text = ref("");
const size = ref(48);
const faceKeys = ref<Record<string, string>>({});

const columns = computed(() =>
  compareIds.value.map((id) => fonts.value.find((f) => f.id === id)).filter((f): f is FontFamilyItem => !!f),
);
// Hasta 3 fuentes: columnas a pantalla completa. De 4 en adelante: cuadros (2×2 con 4; 3×2 con 5 o 6).
const layout = computed(() => compareLayout(columns.value.length));
const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${layout.value.cols}, minmax(${layout.value.kind === "columns" ? 220 : 0}px, 1fr))`,
  gridTemplateRows: `repeat(${layout.value.rows}, minmax(0, 1fr))`,
}));
const rowsOf = (f: FontFamilyItem): FaceRow[] =>
  (f.scanned?.files ?? []).flatMap((file) => file.faces.map((face) => ({ key: `${file.path}:${face.faceIndex}`, file, face })));
const keyOfRow = (file: FamilyFile, face: FaceInfo) => `${file.path}:${face.faceIndex}`;

/** Variante elegida en la columna; por defecto, la representativa (Regular si existe). */
function current(f: FontFamilyItem): FaceRow | null {
  const rows = rowsOf(f);
  const chosen = rows.find((r) => r.key === faceKeys.value[f.id]);
  if (chosen) return chosen;
  const rep = f.scanned ? representativeFace(f.scanned) : null;
  return (rep && rows.find((r) => r.key === keyOfRow(rep.file, rep.face))) || rows[0] || null;
}
const styleFor = (f: FontFamilyItem) => {
  const r = current(f);
  return previewStyle(f.cssFamily, r ? descOf(r.file, r.face) : null, size.value);
};
const styleName = (face: FaceInfo) => (face.isItalic ? "Italic" : face.isOblique ? "Oblique" : "Normal");

function summary(f: FontFamilyItem) {
  const r = current(f);
  if (!f.scanned || !r) return null;
  const cov = faceCoverage(r.face, charSets.value);
  const missing = cov.filter((c) => c.set.category === "spanish").flatMap((c) => c.missing);
  const sym = cov.filter((c) => c.set.category === "symbols");
  const total = sym.reduce((n, c) => n + c.present.length + c.missing.length, 0);
  const have = sym.reduce((n, c) => n + c.present.length, 0);
  const status = faceStatus(r.face, charSets.value);
  return {
    status,
    spanish: missing.length ? `Missing: ${missing.join(" ")}` : "Complete",
    symbols: total ? `${have} of ${total}` : "—",
    weights: familyWeights(f.scanned).map(weightLabel).join(", "),
  };
}
const statusText = { full: "Meets the selected set", partial: "Partial coverage", none: "None of the selected set", unanalyzed: "Not analyzed" } as const;

watch(compareOpen, (open) => { if (open && !text.value) text.value = previewText.value || DEFAULT_TEXT; });
// Se cargan los archivos reales de todas las variantes de las fuentes comparadas.
watch([compareOpen, columns], () => {
  if (!compareOpen.value) return;
  for (const f of columns.value) for (const r of rowsOf(f)) void ensureFace(descOf(r.file, r.face), DETAIL_MAX_BYTES);
}, { immediate: true });
// Si se quitan fuentes y quedan menos de dos, la comparación se cierra.
watch(() => compareIds.value.length, (n) => { if (compareOpen.value && n < 2) closeCompare(); });

const onKey = (e: KeyboardEvent) => { if (compareOpen.value && e.key === "Escape") closeCompare(); };
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div v-if="compareOpen" class="cmp" role="dialog" aria-modal="true" aria-label="Compare fonts">
    <header class="cmp__head">
      <h2>Compare fonts <span class="muted">({{ columns.length }})</span></h2>
      <input v-model="text" class="cmp__text" aria-label="Comparison text" placeholder="Type your own text…" />
      <SampleChips @pick="text = $event" />
      <label class="cmp__size"><span class="muted">Size</span><input v-model.number="size" type="range" min="16" max="120" aria-label="Comparison size" /><output>{{ size }}</output></label>
      <button class="btn" @click="closeCompare">Close</button>
    </header>

    <div class="cmp__cols" :class="`cmp__cols--${layout.kind}`" :style="gridStyle">
      <section v-for="f in columns" :key="f.id" class="cmp__col">
        <header class="cmp__colhead">
          <strong class="cmp__name">{{ f.familyName }}</strong>
          <button class="icon-btn icon-btn--sm" :aria-label="`Remove ${f.familyName} from comparison`" @click="toggleCompare(f.id)">✕</button>
        </header>
        <select v-if="rowsOf(f).length > 1" :value="current(f)?.key" :aria-label="`Variant of ${f.familyName}`"
          @change="faceKeys[f.id] = ($event.target as HTMLSelectElement).value">
          <option v-for="r in rowsOf(f)" :key="r.key" :value="r.key">
            {{ r.face.subfamilyName }} — {{ weightLabel(r.face.weightClass) }}, {{ styleName(r.face) }}
          </option>
        </select>
        <p class="cmp__sample" :style="styleFor(f)">{{ text || DEFAULT_TEXT }}</p>
        <dl v-if="summary(f)" class="cmp__facts">
          <dt>Status</dt><dd><span class="dot" :class="`dot--${summary(f)!.status}`" /> {{ statusText[summary(f)!.status] }}</dd>
          <dt>Spanish</dt><dd>{{ summary(f)!.spanish }}</dd>
          <dt>Symbols</dt><dd>{{ summary(f)!.symbols }}</dd>
          <dt>Weights</dt><dd>{{ summary(f)!.weights }}</dd>
        </dl>
        <p v-else class="muted">Not analyzed (sample data).</p>
      </section>
    </div>
  </div>
</template>
