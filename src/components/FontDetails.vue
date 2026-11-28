<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useLibrary } from "../composables/useLibrary";
import SampleChips from "./SampleChips.vue";
import { SAMPLE_EN } from "../data/sampleTexts";
import { faceCoverage, faceStatus, familyStyles, familyWeights, weightLabel } from "../services/analysis";
import { faceMatchesCoverage, hasCoverageFilters } from "../services/filters";
import { DETAIL_MAX_BYTES, descOf, ensureFace, previewStyle } from "../services/fontFaces";
import type { CharCategory, FaceInfo, FamilyFile } from "../types/scan";

const { selectedFont: font, selectedId, charSets, filters, toggleFavorite, tags, tagsOf, createTag, addTagToFont,
  removeTagFromFont, previewText } = useLibrary();

// ---------- Etiquetas ----------
const fontTags = computed(() => (font.value ? tagsOf(font.value.id) : []));
const unassigned = computed(() => tags.value.filter((t) => !fontTags.value.some((x) => x.id === t.id)));
const pick = ref("");
const newTag = ref("");
const tagError = ref("");
async function guard(action: () => Promise<unknown>) {
  try { await action(); tagError.value = ""; } catch (e) { tagError.value = (e as Error).message; }
}
const addPicked = () => guard(async () => { if (font.value && pick.value) await addTagToFont(font.value.id, Number(pick.value)); pick.value = ""; });
const createAndAdd = () => guard(async () => {
  if (!font.value) return;
  const t = await createTag(newTag.value);
  await addTagToFont(font.value.id, t.id);
  newTag.value = "";
});

// ---------- Variantes ----------
interface FaceRow { key: string; file: FamilyFile; face: FaceInfo }
const faces = computed<FaceRow[]>(() =>
  (font.value?.scanned?.files ?? []).flatMap((file) =>
    file.faces.map((face) => ({ key: `${file.path}:${face.faceIndex}`, file, face })),
  ),
);
const coverageFilterOn = computed(() => hasCoverageFilters(filters.value));
const meets = (r: FaceRow) => coverageFilterOn.value && faceMatchesCoverage(r.face, filters.value, charSets.value);

// ---------- Preview editable ----------
const DEFAULT_TEXT = SAMPLE_EN;
const sample = ref("");
const size = ref(40);

const faceKey = ref("");
watch(() => font.value?.id, () => {
  sample.value = previewText.value || DEFAULT_TEXT;
  const rows = faces.value;
  const regular = rows.find((r) => r.face.weightClass === 400 && !r.face.isItalic && !r.face.isOblique);
  // Si hay filtros de cobertura activos, se preselecciona la variante que los cumple.
  faceKey.value = (rows.find(meets) ?? regular ?? rows[0])?.key ?? "";
}, { immediate: true });

const current = computed(() => faces.value.find((r) => r.key === faceKey.value) ?? null);
const coverage = computed(() => (current.value ? faceCoverage(current.value.face, charSets.value) : []));

const styleName = (f: FaceInfo) => (f.isItalic ? "Italic" : f.isOblique ? "Oblique" : "Normal");
const descFor = (r: FaceRow) => descOf(r.file, r.face);
// Cada variante se dibuja con su archivo real; mientras carga (o si falla) se usa familia + peso + estilo.
const faceCss = (r: FaceRow | null, px?: number) => previewStyle(font.value?.cssFamily ?? "", r ? descFor(r) : null, px);
watch(faces, (rows) => rows.forEach((r) => void ensureFace(descFor(r), DETAIL_MAX_BYTES)), { immediate: true });
const css = computed(() => faceCss(current.value));

// ---------- Resúmenes ----------
const categories: { id: CharCategory; title: string }[] = [
  { id: "latin", title: "Latin letters" },
  { id: "spanish", title: "Spanish" },
  { id: "symbols", title: "Symbols" },
];
const weights = computed(() => (font.value?.scanned ? familyWeights(font.value.scanned).map(weightLabel) : []));
const styles = computed(() => {
  if (!font.value?.scanned) return [];
  const s = familyStyles(font.value.scanned);
  return ["Normal", ...(s.italic ? ["Italic"] : []), ...(s.oblique ? ["Oblique"] : [])];
});
const axes = computed(() =>
  faces.value.flatMap((r) => r.face.axes).filter((a, i, all) => all.findIndex((b) => b.tag === a.tag) === i),
);
const internalNames = computed(() => [...new Set(faces.value.map((r) => r.face.fullName).filter((n): n is string => !!n))]);

const spanishSummary = computed(() => {
  const sp = coverage.value.filter((c) => c.set.category === "spanish");
  if (!sp.length) return "";
  const missing = sp.flatMap((c) => c.missing);
  return missing.length ? `Missing: ${missing.join(" ")}` : "Complete: accents, ñ, ü, ¿ ¡";
});
const symbolSummary = computed(() => {
  const sy = coverage.value.filter((c) => c.set.category === "symbols");
  const total = sy.reduce((n, c) => n + c.present.length + c.missing.length, 0);
  const have = sy.reduce((n, c) => n + c.present.length, 0);
  return total ? `${have} of ${total} special symbols` : "";
});
const statusLabel = { full: "Meets the selected set", partial: "Partial coverage", none: "None of the selected set", unanalyzed: "Not analyzed" } as const;
const kb = (n: number) => (n >= 1048576 ? `${(n / 1048576).toFixed(1)} MB` : `${Math.round(n / 1024)} KB`);
</script>

<template>
  <aside v-if="font" class="details" aria-label="Font details">
    <header class="details__head">
      <h2>{{ font.familyName }}</h2>
      <span class="details__actions">
        <button class="btn" :aria-pressed="font.isFavorite" @click="toggleFavorite(font.id)">
          {{ font.isFavorite ? "★ Favorite" : "☆ Add to favorites" }}
        </button>
        <button class="icon-btn" aria-label="Close details" @click="selectedId = null">✕</button>
      </span>
    </header>

    <p v-if="!font.scanned" class="details__note">Not analyzed. This is sample data, so there are no coverage results to show.</p>

    <template v-else>
      <h3>Preview</h3>
      <div class="preview">
        <input v-model="sample" class="preview__text" aria-label="Preview text" placeholder="Type your own text…" />
        <SampleChips @pick="sample = $event" />
        <label class="preview__size"><span class="muted">Size</span><input v-model.number="size" type="range" min="16" max="120" aria-label="Preview size" /><output>{{ size }}</output></label>
      </div>
      <ul class="variants" aria-label="Variants">
        <li v-for="r in faces" :key="r.key" class="variant" :class="{ 'variant--match': meets(r), 'variant--sel': r.key === faceKey }" @click="faceKey = r.key">
          <span class="variant__meta">
            {{ r.face.subfamilyName }} · {{ weightLabel(r.face.weightClass) }} · {{ styleName(r.face) }}<template v-if="r.face.isVariable"> · variable</template>
            <strong v-if="meets(r)" class="variant__ok"> ✓ meets your filters</strong>
          </span>
          <span class="variant__sample" :style="faceCss(r, size)">{{ sample || DEFAULT_TEXT }}</span>
        </li>
      </ul>

      <h3>Summary</h3>
      <dl class="details__facts">
        <dt>Weights</dt><dd>{{ weights.join(", ") }}</dd>
        <dt>Styles</dt><dd>{{ styles.join(", ") }}</dd>
        <dt>Variable</dt>
        <dd>
          <template v-if="font.isVariable">Yes — <span v-for="a in axes" :key="a.tag" class="axis">{{ a.tag }} {{ a.minValue }}–{{ a.maxValue }}</span></template>
          <template v-else>No</template>
        </dd>
        <dt>Spanish</dt><dd>{{ spanishSummary }}</dd>
        <dt>Symbols</dt><dd>{{ symbolSummary }}</dd>
        <dt v-if="internalNames.length">Names</dt><dd v-if="internalNames.length">{{ internalNames.join(" · ") }}</dd>
      </dl>
    </template>

    <h3>Tags</h3>
    <div class="details__tags">
      <span v-for="t in fontTags" :key="t.id" class="tagchip">
        {{ t.name }}
        <button :aria-label="`Remove tag ${t.name}`" @click="guard(() => removeTagFromFont(font!.id, t.id))">✕</button>
      </span>
      <span v-if="!fontTags.length" class="muted">No tags yet</span>
    </div>
    <div class="details__addtag">
      <select v-if="unassigned.length" v-model="pick" aria-label="Add an existing tag" @change="addPicked">
        <option value="" disabled>Add tag…</option>
        <option v-for="t in unassigned" :key="t.id" :value="t.id">{{ t.name }}</option>
      </select>
      <input v-model="newTag" placeholder="New tag + Enter" aria-label="Create and add a new tag" @keyup.enter="createAndAdd" />
    </div>
    <p v-if="tagError" class="scan__err" role="alert">{{ tagError }}</p>

    <template v-if="font.scanned">
      <h3>Files</h3>
      <ul class="details__files">
        <li v-for="f in font.scanned.files" :key="f.path">
          <strong>{{ f.fileName }}</strong>
          <span class="muted"> · {{ f.format }} · {{ kb(f.size) }} · {{ f.source === "user" ? "installed for user" : "system" }}</span>
          <span class="scan__path">{{ f.path }}</span>
        </li>
      </ul>

      <h3>Coverage</h3>
      <label class="details__face">
        <span>Variant</span>
        <select v-model="faceKey">
          <option v-for="r in faces" :key="r.key" :value="r.key">
            {{ meets(r) ? "✓ " : "" }}{{ r.face.subfamilyName }} — {{ weightLabel(r.face.weightClass) }}, {{ styleName(r.face) }}{{ r.face.isVariable ? ", variable" : "" }}
          </option>
        </select>
      </label>
      <p v-if="current" class="muted">{{ current.face.glyphCount }} glyphs · {{ statusLabel[faceStatus(current.face, charSets)] }}</p>

      <section v-for="cat in categories" :key="cat.id" class="cov">
        <h4>{{ cat.title }}</h4>
        <div v-for="c in coverage.filter((x) => x.set.category === cat.id)" :key="c.set.id" class="cov__row">
          <div class="cov__label">{{ c.set.label }} <span class="muted">{{ c.present.length }}/{{ c.present.length + c.missing.length }}</span></div>
          <div class="cov__chars">
            <span v-for="ch in c.present" :key="ch" class="glyph" :style="css">{{ ch }}</span>
            <span v-for="ch in c.missing" :key="ch" class="glyph glyph--missing" title="Missing in this variant">{{ ch }}</span>
          </div>
        </div>
      </section>
      <p class="muted">Crossed-out characters are missing from this variant. Variants of one family usually share the same coverage.</p>
    </template>
  </aside>
</template>
