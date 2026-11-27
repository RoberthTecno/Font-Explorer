<script setup lang="ts">
import { computed } from "vue";
import { isTauri } from "../services/fontService";
import { useLibrary } from "../composables/useLibrary";

const { lastScan, scanning, rescan, previewSize, previewText, viewMode, resetPreferences } = useLibrary();
const when = computed(() => (lastScan.value ? new Date(lastScan.value.scannedAt * 1000).toLocaleString() : "Never"));
</script>

<template>
  <section class="settings">
    <h2>Preview defaults</h2>
    <label class="settings__row">
      <span>Text size</span>
      <input v-model.number="previewSize" type="range" min="16" max="72" aria-label="Default preview size" />
      <output>{{ previewSize }} px</output>
    </label>
    <label class="settings__row">
      <span>Text</span>
      <input v-model="previewText" type="text" placeholder="Empty = show each font's name" aria-label="Default preview text" />
    </label>
    <label class="settings__row">
      <span>Layout</span>
      <select v-model="viewMode" aria-label="Default layout"><option value="grid">Grid</option><option value="list">List</option></select>
    </label>
    <button class="btn" @click="resetPreferences">Reset to defaults</button>
    <p class="settings__note">{{ isTauri ? "These preferences are saved automatically." : "Browser preview: preferences are not saved." }}</p>

    <h2>Fonts</h2>
    <dl>
      <dt>Last scan</dt>
      <dd>{{ when }}<template v-if="lastScan"> · {{ (lastScan.durationMs / 1000).toFixed(1) }} s</template></dd>
      <dt>Files read</dt>
      <dd>
        {{ lastScan ? `${lastScan.filesOk} of ${lastScan.filesTotal} (${lastScan.filesSkipped} unsupported format skipped)` : "—" }}
        <template v-if="lastScan?.filesReused"> · {{ lastScan.filesReused }} unchanged, not re-read</template>
      </dd>
    </dl>
    <button class="btn" :disabled="!isTauri || scanning" @click="rescan">Rescan fonts</button>

    <details v-if="lastScan?.errors.length" class="settings__errors">
      <summary>View scan errors ({{ lastScan.errors.length }})</summary>
      <ul class="scan__list">
        <li v-for="e in lastScan.errors" :key="e.path">
          <strong>{{ e.fileName }}</strong> — {{ e.kind }}: {{ e.message }}
          <span class="scan__path">{{ e.path }}</span>
        </li>
      </ul>
    </details>
  </section>
</template>
