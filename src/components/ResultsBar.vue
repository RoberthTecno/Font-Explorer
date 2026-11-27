<script setup lang="ts">
import { computed } from "vue";
import { useLibrary } from "../composables/useLibrary";
import { WEIGHT_NAMES } from "../services/analysis";
import { resultsLabel } from "../services/filters";

const { filters, tags, visibleFonts, totalFonts, section, activeFilterCount, unanalyzedExcluded, clearFilters } = useLibrary();

const flags: Record<string, string> = {
  italic: "Italic", oblique: "Oblique", upper: "A–Z", lower: "a–z", accents: "Accents",
  spanish: "Spanish complete", symbols: "All symbols", favorites: "Favorites",
};
const chips = computed(() => {
  const f = filters.value;
  const out: { label: string; remove: () => void }[] = [];
  for (const w of f.weights) out.push({ label: WEIGHT_NAMES[w], remove: () => (f.weights = f.weights.filter((x) => x !== w)) });
  for (const k of Object.keys(flags)) {
    if ((f as unknown as Record<string, boolean>)[k]) out.push({ label: flags[k], remove: () => ((f as unknown as Record<string, boolean>)[k] = false) });
  }
  for (const id of f.tagIds) {
    out.push({ label: `# ${tags.value.find((t) => t.id === id)?.name ?? "tag"}`, remove: () => (f.tagIds = f.tagIds.filter((x) => x !== id)) });
  }
  return out;
});
</script>

<template>
  <div v-if="section.kind !== 'settings'" class="results" role="status" aria-live="polite">
    <span class="results__count" title="Counts font families">{{ resultsLabel(visibleFonts.length, totalFonts) }}</span>
    <span v-for="c in chips" :key="c.label" class="tagchip">
      {{ c.label }}<button :aria-label="`Remove filter ${c.label}`" @click="c.remove">✕</button>
    </span>
    <button v-if="activeFilterCount" class="btn btn--link" @click="clearFilters">Clear all</button>
    <span v-if="unanalyzedExcluded" class="muted">· {{ unanalyzedExcluded }} not analyzed, excluded</span>
  </div>
</template>
