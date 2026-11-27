<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useLibrary } from "../composables/useLibrary";

const { contextMenu: menu, closeContextMenu: close, fonts, sortedTags, tagsOf, toggleFavorite, openFont,
  createTag, addTagToFont, removeTagFromFont, compareIds, toggleCompare } = useLibrary();

const font = computed(() => fonts.value.find((f) => f.id === menu.value?.fontId) ?? null);
const assigned = computed(() => new Set(font.value ? tagsOf(font.value.id).map((t) => t.id) : []));
const el = ref<HTMLElement | null>(null);
const pos = ref({ left: 0, top: 0 });
const newTag = ref("");
const error = ref("");

// Se coloca en el cursor y se ajusta para que nunca se salga de la ventana.
watch(menu, async (m) => {
  newTag.value = ""; error.value = "";
  if (!m) return;
  pos.value = { left: m.x, top: m.y };
  await nextTick();
  const r = el.value?.getBoundingClientRect();
  if (r) pos.value = {
    left: Math.max(8, Math.min(m.x, window.innerWidth - r.width - 8)),
    top: Math.max(8, Math.min(m.y, window.innerHeight - r.height - 8)),
  };
  el.value?.querySelector<HTMLElement>("button")?.focus();
});

async function toggleTag(tagId: number) {
  if (!font.value) return;
  try {
    if (assigned.value.has(tagId)) await removeTagFromFont(font.value.id, tagId);
    else await addTagToFont(font.value.id, tagId);
    error.value = "";
  } catch (e) { error.value = (e as Error).message; }
}

async function createAndAssign() {
  if (!font.value) return;
  try {
    const t = await createTag(newTag.value);
    await addTagToFont(font.value.id, t.id);
    newTag.value = ""; error.value = "";
  } catch (e) { error.value = (e as Error).message; }
}

function onKey(e: KeyboardEvent) {
  if (!menu.value) return;
  if (e.key === "Escape") { close(); return; }
  if ((e.key === "ArrowDown" || e.key === "ArrowUp") && (e.target as HTMLElement).tagName === "BUTTON") {
    e.preventDefault();
    const items = [...(el.value?.querySelectorAll<HTMLElement>("button") ?? [])];
    const i = items.indexOf(e.target as HTMLElement);
    items[(i + (e.key === "ArrowDown" ? 1 : -1) + items.length) % items.length]?.focus();
  }
}
const outside = (e: Event) => { if (menu.value && !el.value?.contains(e.target as Node)) close(); };

onMounted(() => {
  window.addEventListener("pointerdown", outside, true);
  window.addEventListener("keydown", onKey);
  window.addEventListener("resize", close);
  window.addEventListener("blur", close);
  window.addEventListener("scroll", close, true);
});
onBeforeUnmount(() => {
  window.removeEventListener("pointerdown", outside, true);
  window.removeEventListener("keydown", onKey);
  window.removeEventListener("resize", close);
  window.removeEventListener("blur", close);
  window.removeEventListener("scroll", close, true);
});
</script>

<template>
  <div v-if="menu && font" ref="el" class="ctx" role="menu" :aria-label="`Actions for ${font.familyName}`"
    :style="{ left: pos.left + 'px', top: pos.top + 'px' }" @contextmenu.prevent>
    <p class="ctx__title">{{ font.familyName }}</p>
    <button class="ctx__item" role="menuitem" @click="toggleFavorite(font.id); close()">
      {{ font.isFavorite ? "★ Remove from favorites" : "☆ Add to favorites" }}
    </button>
    <button class="ctx__item" role="menuitem" @click="openFont(font.id); close()">Show details…</button>
    <button class="ctx__item" role="menuitem" @click="toggleCompare(font.id); close()">
      {{ compareIds.includes(font.id) ? "Remove from comparison" : "Add to comparison" }}
    </button>
    <hr />
    <p class="ctx__label">Tags</p>
    <div class="ctx__tags">
      <button v-for="t in sortedTags" :key="t.id" class="ctx__item" role="menuitemcheckbox" :aria-checked="assigned.has(t.id)" @click="toggleTag(t.id)">
        <span class="ctx__check" aria-hidden="true">{{ assigned.has(t.id) ? "✓" : "" }}</span>{{ t.name }}
      </button>
      <p v-if="!sortedTags.length" class="ctx__empty">No tags yet — create one below.</p>
    </div>
    <input v-model="newTag" class="ctx__input" placeholder="New tag + Enter" aria-label="Create a new tag and add it to this font"
      @keyup.enter="createAndAssign" />
    <p v-if="error" class="ctx__error" role="alert">{{ error }}</p>
  </div>
</template>
