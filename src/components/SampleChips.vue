<script setup lang="ts">
import { SAMPLE_EN, SAMPLE_ES } from "../data/sampleTexts";
defineEmits<{ pick: [text: string] }>();
</script>

<template>
  <span class="samples" role="group" aria-label="Sample texts">
    <button class="fchip" @click="$emit('pick', SAMPLE_EN)">English</button>
    <button class="fchip" @click="$emit('pick', SAMPLE_ES)">Español</button>
  </span>
</template>
