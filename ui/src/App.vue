<script setup lang="ts">
// Step 8 page: proves the backend wiring by rendering status and catalog. Step 9 replaces it
// with the real shell.
import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { commands, type CatalogDto, type StatusDto } from "./bindings";
import { call, errorMessage } from "./lib/api";

const status = ref<StatusDto | null>(null);
const catalog = ref<CatalogDto | null>(null);
const error = ref<string | null>(null);
let unlisten: UnlistenFn | undefined;

async function load() {
  try {
    status.value = await call(commands.getStatus());
    catalog.value = await call(commands.getCatalog());
    error.value = null;
  } catch (e) {
    error.value = errorMessage(e);
  }
}

onMounted(async () => {
  unlisten = await listen("data-changed", load);
  await load();
});
onUnmounted(() => unlisten?.());
</script>

<template>
  <main>
    <h1>ThorSoft TimeTracker</h1>
    <p v-if="error" class="error">{{ error }}</p>
    <h2>Status</h2>
    <pre>{{ JSON.stringify(status, null, 2) }}</pre>
    <h2>Catalog</h2>
    <pre>{{ JSON.stringify(catalog, null, 2) }}</pre>
  </main>
</template>

<style>
body {
  font-family: "Segoe UI", system-ui, sans-serif;
  margin: 2rem;
}
.error {
  color: #b00020;
}
</style>
