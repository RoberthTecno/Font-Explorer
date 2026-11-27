<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import FilterPanel from "./FilterPanel.vue";
import { isTauri } from "../services/fontService";
import Icon from "./Icon.vue";
import { useLibrary } from "../composables/useLibrary";

const { viewMode, previewSize, search, sidebarOpen, section, sectionTitle, activeFilterCount, startImport, startExport, exportTargets } = useLibrary();
const legendOpen = ref(false);
const filtersOpen = ref(false);
const filtersWrap = ref<HTMLElement | null>(null);
const legendWrap = ref<HTMLElement | null>(null);
const exportOpen = ref(false);
const exportWrap = ref<HTMLElement | null>(null);
function doExport(scope: "shown" | "selected") { exportOpen.value = false; void startExport(scope); }

// Cierra los paneles al hacer clic fuera o pulsar Escape.
const outside = (e: PointerEvent) => {
  if (filtersOpen.value && !filtersWrap.value?.contains(e.target as Node)) filtersOpen.value = false;
  if (legendOpen.value && !legendWrap.value?.contains(e.target as Node)) legendOpen.value = false;
  if (exportOpen.value && !exportWrap.value?.contains(e.target as Node)) exportOpen.value = false;
};
const esc = (e: KeyboardEvent) => { if (e.key === "Escape") { filtersOpen.value = false; legendOpen.value = false; exportOpen.value = false; } };
onMounted(() => { window.addEventListener("pointerdown", outside); window.addEventListener("keydown", esc); });
onBeforeUnmount(() => { window.removeEventListener("pointerdown", outside); window.removeEventListener("keydown", esc); });
</script>

<template>
  <header class="topbar">
    <button class="icon-btn" :aria-pressed="sidebarOpen" aria-label="Toggle sidebar" @click="sidebarOpen = !sidebarOpen">
      <Icon name="sidebar" />
    </button>
    <h1 class="topbar__title">{{ sectionTitle(section) }}</h1>

    <label class="size" title="Preview size">
      <span class="size__a size__a--s" aria-hidden="true">A</span>
      <input v-model.number="previewSize" type="range" min="16" max="72" step="1" aria-label="Preview text size" />
      <span class="size__a size__a--l" aria-hidden="true">A</span>
    </label>

    <div class="topbar__spacer" />

    <div class="segmented" role="group" aria-label="Layout">
      <button :class="{ on: viewMode === 'grid' }" :aria-pressed="viewMode === 'grid'" aria-label="Grid view" @click="viewMode = 'grid'">
        <Icon name="grid" />
      </button>
      <button :class="{ on: viewMode === 'list' }" :aria-pressed="viewMode === 'list'" aria-label="List view" @click="viewMode = 'list'">
        <Icon name="list" />
      </button>
    </div>

    <button class="btn btn--pill" :disabled="!isTauri" :title="isTauri ? 'Install font files (.ttf, .otf) for your Windows user' : 'Available in the desktop app'" @click="startImport">
      Import
    </button>
    <div ref="exportWrap" class="legend-wrap">
      <button class="btn btn--pill" :disabled="!isTauri" :aria-expanded="exportOpen" :title="isTauri ? 'Copy font files to a folder' : 'Available in the desktop app'" @click="exportOpen = !exportOpen">
        Export
      </button>
      <div v-if="exportOpen" class="legend" role="menu" aria-label="Export">
        <button class="ctx__item" role="menuitem" :disabled="!exportTargets('shown').length" @click="doExport('shown')">
          Fonts shown now ({{ exportTargets('shown').length }})
        </button>
        <button class="ctx__item" role="menuitem" :disabled="!exportTargets('selected').length" @click="doExport('selected')">
          Selected fonts ({{ exportTargets('selected').length }})
        </button>
        <p class="legend__note">Files are copied into one folder per family. Nothing is overwritten.</p>
      </div>
    </div>

    <div ref="filtersWrap" class="legend-wrap">
      <button class="btn btn--pill" :class="{ on: activeFilterCount > 0 }" :aria-expanded="filtersOpen" @click="filtersOpen = !filtersOpen">
        <Icon name="filter" :size="15" /> Filters<span v-if="activeFilterCount" class="badge">{{ activeFilterCount }}</span>
      </button>
      <FilterPanel v-if="filtersOpen" />
    </div>

    <div ref="legendWrap" class="legend-wrap">
      <button class="icon-btn" :aria-expanded="legendOpen" aria-label="Status legend" @click="legendOpen = !legendOpen">
        <Icon name="info" />
      </button>
      <div v-if="legendOpen" class="legend" role="dialog" aria-label="Status legend">
        <p class="legend__title">Analysis status</p>
        <p class="legend__note">Selected set: A–Z, a–z and Spanish characters (á é í ó ú ñ ü ¿ ¡).</p>
        <p><span class="dot dot--full" /> Meets the selected character set</p>
        <p><span class="dot dot--partial" /> Partial coverage</p>
        <p><span class="dot dot--none" /> None of the selected set</p>
        <p><span class="dot dot--unanalyzed" /> Not analyzed or unavailable</p>
        <p class="legend__note">Sample data (browser preview) is never analyzed.</p>
      </div>
    </div>

    <label class="search">
      <Icon name="search" :size="16" />
      <input v-model="search" type="search" placeholder="Search" aria-label="Search fonts by name" />
    </label>
  </header>
</template>
