<script setup lang="ts">
import { ref } from "vue";
import Icon from "./Icon.vue";
import { useLibrary } from "../composables/useLibrary";

const { sortedTags, section, createTag, renameTag, deleteTag } = useLibrary();
const vFocus = { mounted: (el: HTMLElement) => el.focus() };

const creating = ref(false);
const newName = ref("");
const editingId = ref<number | null>(null);
const editName = ref("");
const confirmId = ref<number | null>(null);
const error = ref("");

async function run(action: () => Promise<unknown>, done: () => void) {
  try { await action(); error.value = ""; done(); } catch (e) { error.value = (e as Error).message; }
}
const submitNew = () => run(() => createTag(newName.value), () => { newName.value = ""; creating.value = false; });
const submitEdit = () => run(() => renameTag(editingId.value!, editName.value), () => { editingId.value = null; });
const confirmDelete = (id: number) => run(() => deleteTag(id), () => { confirmId.value = null; });
function startEdit(id: number, name: string) { editingId.value = id; editName.value = name; confirmId.value = null; error.value = ""; }
function cancelAll() { creating.value = false; editingId.value = null; confirmId.value = null; error.value = ""; }
</script>

<template>
  <div class="taglist">
    <div class="taglist__head">
      <h2 class="sidebar__heading">Tags</h2>
      <button class="icon-btn icon-btn--sm" aria-label="New tag" @click="creating = true; error = ''"><Icon name="plus" :size="15" /></button>
    </div>
    <input v-if="creating" v-model="newName" v-focus class="taglist__input" placeholder="Tag name" aria-label="New tag name"
      @keyup.enter="submitNew" @keyup.esc="cancelAll" />
    <p v-if="error" class="taglist__error" role="alert">{{ error }}</p>

    <ul class="nav nav--scroll">
      <li v-for="t in sortedTags" :key="t.id" class="tagrow">
        <input v-if="editingId === t.id" v-model="editName" v-focus class="taglist__input" aria-label="Tag name"
          @keyup.enter="submitEdit" @keyup.esc="cancelAll" />
        <div v-else-if="confirmId === t.id" class="tagrow__confirm" role="alertdialog">
          <span>Delete “{{ t.name }}”?</span>
          <button class="btn" @click="confirmDelete(t.id)">Delete</button>
          <button class="btn" @click="cancelAll">Keep</button>
        </div>
        <template v-else>
          <button class="nav__item" :class="{ active: section.kind === 'tag' && section.id === t.id }" @click="section = { kind: 'tag', id: t.id }">
            <Icon name="tag" /><span>{{ t.name }}</span>
          </button>
          <span class="tagrow__actions">
            <button class="icon-btn icon-btn--sm" :aria-label="`Rename ${t.name}`" @click="startEdit(t.id, t.name)"><Icon name="edit" :size="14" /></button>
            <button class="icon-btn icon-btn--sm" :aria-label="`Delete ${t.name}`" @click="confirmId = t.id; editingId = null"><Icon name="close" :size="14" /></button>
          </span>
        </template>
      </li>
    </ul>
    <p v-if="!sortedTags.length && !creating" class="taglist__empty">No tags yet. Use + to create one.</p>
  </div>
</template>
