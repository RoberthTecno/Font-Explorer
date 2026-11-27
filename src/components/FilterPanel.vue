<script setup lang="ts">
import { useLibrary } from "../composables/useLibrary";
import { WEIGHT_NAMES } from "../services/analysis";

const { filters, sortedTags, clearFilters, activeFilterCount } = useLibrary();
const weights = Object.keys(WEIGHT_NAMES).map(Number);

function toggle<T>(list: T[], value: T): T[] {
  return list.includes(value) ? list.filter((x) => x !== value) : [...list, value];
}
const toggles: { key: "italic" | "oblique" | "upper" | "lower" | "accents" | "spanish" | "symbols" | "favorites"; label: string; group: string }[] = [
  { key: "italic", label: "Has Italic", group: "Style" },
  { key: "oblique", label: "Has Oblique", group: "Style" },
  { key: "upper", label: "Has uppercase A–Z", group: "Characters" },
  { key: "lower", label: "Has lowercase a–z", group: "Characters" },
  { key: "accents", label: "Has all accents (á é í ó ú)", group: "Characters" },
  { key: "spanish", label: "Complete Spanish set", group: "Characters" },
  { key: "symbols", label: "Has all special symbols", group: "Characters" },
  { key: "favorites", label: "Favorites only", group: "Library" },
];
const groups = ["Style", "Characters", "Library"];
</script>

<template>
  <div class="filters" role="dialog" aria-label="Filters">
    <div class="filters__head">
      <strong>Filters</strong>
      <button class="btn btn--link" :disabled="!activeFilterCount" @click="clearFilters">Clear all</button>
    </div>
    <p class="muted filters__hint">All active filters apply together. A family matches when at least one of its variants does.</p>

    <h4>Weight (family must have all selected)</h4>
    <div class="fchips">
      <button v-for="w in weights" :key="w" class="fchip" :class="{ on: filters.weights.includes(w) }" :aria-pressed="filters.weights.includes(w)"
        @click="filters.weights = toggle(filters.weights, w)">{{ WEIGHT_NAMES[w] }}</button>
    </div>

    <template v-for="g in groups" :key="g">
      <h4>{{ g }}</h4>
      <label v-for="t in toggles.filter((x) => x.group === g)" :key="t.key" class="check">
        <input v-model="filters[t.key]" type="checkbox" /> {{ t.label }}
      </label>
    </template>

    <h4>Tags (family must have all selected)</h4>
    <div class="fchips">
      <button v-for="t in sortedTags" :key="t.id" class="fchip" :class="{ on: filters.tagIds.includes(t.id) }" :aria-pressed="filters.tagIds.includes(t.id)"
        @click="filters.tagIds = toggle(filters.tagIds, t.id)"># {{ t.name }}</button>
      <span v-if="!sortedTags.length" class="muted">No tags yet.</span>
    </div>
  </div>
</template>
