<script setup lang="ts">
// A value to book that is itself the button copying just the number, e.g. "1,25" (R15).
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
  <Button
    :aria-label="`Copy ${value}`"
    :title="copied ? 'Copied' : 'Copy'"
    size="small"
    text
    class="copy-value"
    @click="copy"
  >
    <!-- The check sits left of the number, so the right-aligned number doesn't move. -->
    <span v-if="copied" class="copied" aria-hidden="true">✓</span>
    <span class="value">{{ value }}</span>
  </Button>
</template>

<style scoped>
.copy-value {
  gap: 0.25rem;
}
.value {
  /* The slot replaces the button's label, so the label's weight is set here. */
  font-weight: var(--p-button-label-font-weight);
  font-variant-numeric: tabular-nums;
}
</style>
