<script setup lang="ts">
import { onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";

const input = ref<HTMLInputElement | null>(null);
const query = ref("");

// The window gets focus from Rust; the input only gets it when the page asks (plan step 10).
onMounted(async () => {
  await listen("quick-switch-shown", () => {
    query.value = "";
    input.value?.focus();
  });
  input.value?.focus();
});
</script>

<template>
  <div class="popup">
    <input ref="input" v-model="query" placeholder="Switch to… (Esc closes)" />
  </div>
</template>

<style>
body {
  margin: 0;
  font-family: "Segoe UI", system-ui, sans-serif;
}
.popup {
  padding: 1rem;
}
.popup input {
  width: 100%;
  box-sizing: border-box;
  font-size: 1.2rem;
  padding: 0.5rem;
}
</style>
