<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "./Icon.vue";
import { CARD_MAX_BYTES, descOf, ensureFace, previewStyle } from "../services/fontFaces";
import { representativeFace } from "../services/analysis";
import type { AnalysisStatus, FontFamilyItem } from "../types/font";

const props = defineProps<{ font: FontFamilyItem; text: string; selected: boolean }>();
const emit = defineEmits<{
  toggleFavorite: [id: string]; open: [id: string]; menu: [id: string, x: number, y: number]; select: [id: string];
}>();

const statusLabel: Record<AnalysisStatus, string> = {
  full: "Meets the selected character set",
  partial: "Partial coverage",
  none: "None of the selected set",
  unanalyzed: "Not analyzed",
};

// El preview se dibuja con la variante representativa y su archivo real (se carga al hacerse visible).
const root = ref<HTMLElement | null>(null);
const visible = ref(false);
const desc = computed(() => {
  const r = props.font.scanned ? representativeFace(props.font.scanned) : null;
  return r ? descOf(r.file, r.face) : null;
});
// El tamaño NO va aquí: lo aplica una variable CSS del contenedor (--preview-size), así mover el slider
// no obliga a recalcular cientos de tarjetas.
const style = computed(() => previewStyle(props.font.cssFamily, desc.value));

let io: IntersectionObserver | undefined;
onMounted(() => {
  if (typeof IntersectionObserver === "undefined") { visible.value = true; return; }
  io = new IntersectionObserver((entries) => {
    if (entries.some((e) => e.isIntersecting)) { visible.value = true; io?.disconnect(); }
  }, { rootMargin: "300px" });
  if (root.value) io.observe(root.value);
});
onBeforeUnmount(() => io?.disconnect());
watch([visible, desc], () => {
  if (visible.value && desc.value) void ensureFace(desc.value, CARD_MAX_BYTES);
}, { immediate: true });

function onClick(e: MouseEvent) {
  // Ctrl + clic (o Cmd) alterna la selección para comparar.
  if (e.ctrlKey || e.metaKey) emit("select", props.font.id);
  else emit("open", props.font.id);
}
function keyMenu(e: KeyboardEvent) {
  // Tecla Menú o Shift+F10: abre el mismo menú que el clic derecho, para quien usa teclado.
  if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
    e.preventDefault();
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    emit("menu", props.font.id, r.left + 24, r.top + 24);
  }
}
</script>

<template>
  <article
    ref="root" class="card" :class="{ 'card--selected': selected }" tabindex="0"
    @click="onClick" @keydown.enter="emit('open', font.id)" @keydown="keyMenu"
    @contextmenu.prevent="emit('menu', font.id, $event.clientX, $event.clientY)"
  >
    <header class="card__head">
      <span class="dot" :class="`dot--${font.status}`" role="img" :aria-label="statusLabel[font.status]" :title="statusLabel[font.status]" />
      <span class="card__kind" :title="font.isVariable ? 'Variable font' : 'Static font'">{{ font.isVariable ? "V" : "Tт" }}</span>
      <span class="card__name">{{ font.familyName }}<template v-if="font.variantCount > 1"> ({{ font.variantCount }})</template></span>
      <button
        class="star" :class="{ on: font.isFavorite }" :aria-pressed="font.isFavorite"
        :aria-label="font.isFavorite ? 'Remove from favorites' : 'Add to favorites'"
        @click.stop="emit('toggleFavorite', font.id)"
      >
        <Icon name="star" :size="15" />
      </button>
    </header>
    <p class="card__preview" :style="style">{{ text || font.familyName }}</p>
    <button
      class="pick" :class="{ on: selected }" role="checkbox" :aria-checked="selected"
      :aria-label="selected ? `Remove ${font.familyName} from comparison` : `Add ${font.familyName} to comparison`"
      title="Select to compare (or Ctrl + click)" @click.stop="emit('select', font.id)"
    >
      <Icon v-if="selected" name="check" :size="13" />
    </button>
  </article>
</template>
