<script setup lang="ts">
// A value to book with a button that copies just the number, e.g. "1,25" (R15).
import { onUnmounted, ref } from "vue";
import Button from "primevue/button";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useAttempt } from "../lib/attempt";

const props = defineProps<{ value: string }>();

const attempt = useAttempt();
const copied = ref(false);
let reset: ReturnType<typeof setTimeout> | undefined;

async function copy() {
  if (await attempt(() => writeText(props.value))) {
    copied.value = true;
    clearTimeout(reset);
    reset = setTimeout(() => (copied.value = false), 1500);
  }
}

onUnmounted(() => clearTimeout(reset));
</script>

<template>
  <span class="copy-value">
    <span class="value">{{ value }}</span>
    <Button
      :label="copied ? 'Copied' : 'Copy'"
      :aria-label="`Copy ${value}`"
      size="small"
      text
      class="copy"
      @click="copy"
    />
  </span>
</template>

<style scoped>
.copy-value {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}
.value {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.copy {
  /* Wide enough for "Copied", so the column doesn't jump. */
  min-width: 4.5rem;
}
</style>
