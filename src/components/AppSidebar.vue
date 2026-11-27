<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "./Icon.vue";
import TagList from "./TagList.vue";
import { useLibrary } from "../composables/useLibrary";
import { VARIATIONS, type VariationGroup } from "../services/analysis";
import type { Section } from "../types/font";

const { section, fonts, variationCounts, hasAnalysis } = useLibrary();
const smartOpen = ref(true);

// Solo se muestran las variaciones que existen en tu biblioteca.
const groups = computed(() =>
  (["Weight", "Style", "Width"] as VariationGroup[])
    .map((g) => ({ name: g, items: VARIATIONS.filter((v) => v.group === g && variationCounts.value[v.id] > 0) }))
    .filter((g) => g.items.length > 0),
);

function isActive(s: Section): boolean {
  const c = section.value;
  if (c.kind !== s.kind) return false;
  if (c.kind === "smart" && s.kind === "smart") return c.id === s.id;
  return true;
}
const go = (s: Section) => { section.value = s; };
</script>

<template>
  <nav class="sidebar" aria-label="Library navigation">
    <ul class="nav">
      <li>
        <button class="nav__item" :class="{ active: isActive({ kind: 'all' }) }" @click="go({ kind: 'all' })">
          <Icon name="fonts" /><span>All Fonts</span><span class="nav__count">{{ fonts.length }}</span>
        </button>
      </li>
      <li>
        <button class="nav__item" :class="{ active: isActive({ kind: 'favorites' }) }" @click="go({ kind: 'favorites' })">
          <Icon name="star" /><span>Favorites</span>
        </button>
      </li>
      <li>
        <button class="nav__item" :class="{ active: isActive({ kind: 'recents' }) }" @click="go({ kind: 'recents' })">
          <Icon name="clock" /><span>Recents</span>
        </button>
      </li>
      <li>
        <button class="nav__item" :aria-expanded="smartOpen" @click="smartOpen = !smartOpen">
          <Icon name="filter" /><span>Smart Filters</span>
          <Icon name="chevron" :size="14" :class="['chev', { open: smartOpen }]" />
        </button>
        <div v-if="smartOpen" class="smart">
          <p v-if="!hasAnalysis" class="taglist__empty">Variations appear after your fonts are analyzed.</p>
          <template v-for="g in groups" :key="g.name">
            <p class="nav__group">{{ g.name }}</p>
            <ul class="nav nav--sub">
              <li v-for="v in g.items" :key="v.id">
                <button class="nav__item" :class="{ active: isActive({ kind: 'smart', id: v.id }) }" @click="go({ kind: 'smart', id: v.id })">
                  <span>{{ v.label }}</span><span class="nav__count">{{ variationCounts[v.id] }}</span>
                </button>
              </li>
            </ul>
          </template>
        </div>
      </li>
    </ul>

    <TagList />

    <div class="sidebar__foot">
      <button class="nav__item" :class="{ active: isActive({ kind: 'settings' }) }" @click="go({ kind: 'settings' })">
        <Icon name="settings" /><span>Settings</span>
      </button>
    </div>
  </nav>
</template>
