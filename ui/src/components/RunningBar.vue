<script setup lang="ts">
// The running entry with its elapsed time, and Stop (R5).
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import { useTimerStore } from "../stores/timer";
import { useAttempt } from "../lib/attempt";
import { formatElapsed } from "../lib/format";

const timer = useTimerStore();
const { running, elapsed } = storeToRefs(timer);
const attempt = useAttempt();
</script>

<template>
  <div class="running-bar" :class="{ active: running }">
    <template v-if="running">
      <span class="dot" aria-hidden="true"></span>
      <span class="label" :title="running.combo.label">{{ running.combo.label }}</span>
      <span class="elapsed">{{ formatElapsed(elapsed) }}</span>
      <Button label="Stop" severity="danger" size="small" @click="attempt(timer.stop)" />
    </template>
    <span v-else class="muted">No timer running</span>
  </div>
</template>

<style scoped>
.running-bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  /* Fixed, so the page doesn't jump when a timer starts or stops. */
  height: 2.75rem;
  box-sizing: border-box;
  padding: 0 1rem;
  border-bottom: 1px solid var(--p-content-border-color);
}
.running-bar.active {
  background: color-mix(in srgb, var(--p-primary-color) 10%, transparent);
}
.dot {
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 50%;
  background: var(--p-primary-color);
}
.label {
  flex: 1;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.elapsed {
  font-variant-numeric: tabular-nums;
  font-size: 1.15rem;
}
</style>
