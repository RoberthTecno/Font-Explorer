<script setup lang="ts">
import { useLibrary } from "../composables/useLibrary";
import { previewChips } from "../data/mockFonts";

const { previewText } = useLibrary();
</script>

<template>
  <div class="previewbar">
    <input v-model="previewText" type="text" placeholder="Enter preview text…" aria-label="Preview text" />
    <div class="chips">
      <button v-for="c in previewChips" :key="c.label" class="chip" @click="previewText = c.text">{{ c.label }}</button>
      <button class="chip chip--clear" :disabled="!previewText" @click="previewText = ''">Clear</button>
    </div>
  </div>
</template>
