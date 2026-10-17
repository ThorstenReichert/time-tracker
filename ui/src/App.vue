<script setup lang="ts">
import { onMounted, ref } from "vue";
import { commands, type Pong } from "./bindings";

const pong = ref<Pong | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    pong.value = await commands.ping();
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <main>
    <h1>ThorSoft TimeTracker</h1>
    <p v-if="pong">Backend says: {{ pong.message }} (build {{ pong.buildId }})</p>
    <p v-else-if="error" class="error">{{ error }}</p>
    <p v-else>Connecting…</p>
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
