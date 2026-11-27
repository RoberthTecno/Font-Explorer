<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useLibrary } from "../composables/useLibrary";

const { transfer, confirmImport, closeTransfer, scanning } = useLibrary();

const names = computed(() => (transfer.value?.paths ?? []).map((p) => p.split(/[\\/]/).pop() ?? p));
const shown = computed(() => names.value.slice(0, 8));
const pct = computed(() => {
  const p = transfer.value?.progress;
  return p && p.total ? Math.round((p.processed / p.total) * 100) : 0;
});
const title = computed(() => (transfer.value?.kind === "import" ? "Import fonts" : "Export fonts"));
const verb = computed(() => (transfer.value?.kind === "import" ? "Installed" : "Exported"));
// Lo que no salió bien o se omitió, para explicarlo; lo correcto se resume en el contador.
const problems = computed(() => (transfer.value?.result?.items ?? []).filter((i) => i.status !== "done"));

const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") closeTransfer(); };
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div v-if="transfer" class="modal" role="dialog" aria-modal="true" :aria-label="title">
    <div class="modal__box">
      <h2>{{ title }}</h2>

      <template v-if="transfer.phase === 'confirm'">
        <p>Install <strong>{{ transfer.paths.length }}</strong> font file{{ transfer.paths.length === 1 ? "" : "s" }} for your Windows user?</p>
        <ul class="modal__list">
          <li v-for="n in shown" :key="n">{{ n }}</li>
          <li v-if="names.length > shown.length" class="muted">…and {{ names.length - shown.length }} more</li>
        </ul>
        <p class="muted">
          The files are copied to your user fonts folder and registered with Windows. No administrator is needed,
          and the original files stay where they are.
        </p>
        <div class="modal__actions">
          <button class="btn" @click="closeTransfer">Cancel</button>
          <button class="btn btn--primary" @click="confirmImport">Install</button>
        </div>
      </template>

      <template v-else-if="transfer.phase === 'running'">
        <p>{{ transfer.kind === "import" ? "Installing fonts…" : "Copying font files…" }}</p>
        <progress :value="pct" max="100" aria-label="Progress" />
        <p class="muted">{{ transfer.progress?.processed ?? 0 }} of {{ transfer.progress?.total ?? "…" }} files</p>
      </template>

      <template v-else-if="transfer.phase === 'done' && transfer.result">
        <p>
          <strong>{{ verb }} {{ transfer.result.done }}</strong>
          <template v-if="transfer.result.skipped"> · {{ transfer.result.skipped }} skipped</template>
          <template v-if="transfer.result.failed"> · <span class="scan__err">{{ transfer.result.failed }} failed</span></template>
        </p>
        <p class="muted modal__folder">{{ transfer.result.folder }}</p>
        <ul v-if="problems.length" class="modal__list modal__list--scroll">
          <li v-for="(i, k) in problems" :key="k"><strong>{{ i.name }}</strong> — {{ i.message }}</li>
        </ul>
        <p v-if="transfer.kind === 'import' && transfer.result.done" class="muted">
          {{ scanning ? "Updating your library…" : "Your library has been updated." }} Apps that were already open may need a restart to see the new fonts.
        </p>
        <div class="modal__actions"><button class="btn btn--primary" @click="closeTransfer">Close</button></div>
      </template>

      <template v-else>
        <p class="scan__err" role="alert">{{ transfer.error || "Something went wrong." }}</p>
        <div class="modal__actions"><button class="btn" @click="closeTransfer">Close</button></div>
      </template>
    </div>
  </div>
</template>
