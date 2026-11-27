<script setup lang="ts">
import { onMounted } from "vue";
import AppSidebar from "./components/AppSidebar.vue";
import TopBar from "./components/TopBar.vue";
import FontCard from "./components/FontCard.vue";
import PreviewBar from "./components/PreviewBar.vue";
import ScanStatus from "./components/ScanStatus.vue";
import ResultsBar from "./components/ResultsBar.vue";
import ContextMenu from "./components/ContextMenu.vue";
import SelectionBar from "./components/SelectionBar.vue";
import TransferDialog from "./components/TransferDialog.vue";
import ComparisonView from "./components/ComparisonView.vue";
import FontDetails from "./components/FontDetails.vue";
import SettingsView from "./components/SettingsView.vue";
import { useLibrary } from "./composables/useLibrary";
import { checkForUpdates } from "./services/updateService.js";

const { visibleFonts, viewMode, previewSize, previewText, sidebarOpen, section, toggleFavorite, scanning, initScan,
  openFont, openContextMenu, selectedFont, compareIds, toggleCompare } = useLibrary();
onMounted(initScan);


const result = await checkForUpdates();

if (result.available) {
  console.log('Nueva versión:', result.version);
  //await result.install();
}


</script>



<template>
  <!-- El detalle es una tercera columna: al abrirse empuja la biblioteca y ninguna tarjeta queda tapada. -->
  <div class="app" :class="{ 'app--no-sidebar': !sidebarOpen, 'app--details': !!selectedFont }">
    <AppSidebar v-if="sidebarOpen" />
    <div class="main">
      <TopBar />
      <ScanStatus />
      <ResultsBar />
      <main class="content">
        <SettingsView v-if="section.kind === 'settings'" />
        <template v-else>
          <section v-if="visibleFonts.length === 0" class="empty">
            <template v-if="scanning">
              <h2>Scanning your fonts…</h2>
              <p>They will appear here as soon as the scan finishes.</p>
            </template>
            <template v-else>
              <h2>No fonts here</h2>
              <p>Try another search or pick a different section from the sidebar.</p>
            </template>
          </section>
          <div v-else class="library" :style="{ '--preview-size': previewSize + 'px' }" :class="[viewMode === 'list' ? 'library--list' : 'library--grid', { 'library--selecting': compareIds.length > 0 }]">
            <FontCard
              v-for="f in visibleFonts" :key="f.id" :font="f" :text="previewText"
              :selected="compareIds.includes(f.id)"
              @toggle-favorite="toggleFavorite" @open="openFont" @menu="openContextMenu" @select="toggleCompare"
            />
          </div>
        </template>
      </main>
      <SelectionBar />
      <PreviewBar v-if="section.kind !== 'settings'" />
    </div>
    <FontDetails />
    <ContextMenu />
    <ComparisonView />
    <TransferDialog />
  </div>
</template>
