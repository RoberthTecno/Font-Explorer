<script setup lang="ts">
import { useLibrary } from "../composables/useLibrary";
import { isTauri } from "../services/fontService";
const { compareIds, compareNotice, openCompare, clearCompare, startExport } = useLibrary();
</script>

<template>
  <div v-if="compareIds.length" class="selbar" role="region" aria-label="Comparison selection">
    <span><strong>{{ compareIds.length }}</strong> selected</span>
    <button class="btn btn--primary" :disabled="compareIds.length < 2" @click="openCompare">Compare</button>
    <button class="btn" :disabled="!isTauri" :title="isTauri ? 'Copy the selected fonts to a folder' : 'Available in the desktop app'" @click="startExport('selected')">Export</button>
    <button class="btn" @click="clearCompare">Clear</button>
    <span v-if="compareIds.length < 2" class="muted">Select at least 2 fonts</span>
    <span v-if="compareNotice" class="scan__err" role="alert">{{ compareNotice }}</span>
  </div>
</template>
