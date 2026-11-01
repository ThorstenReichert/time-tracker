<script setup lang="ts">
// The running entry with its elapsed time, and Stop (R5). Edit and "Stop at…" fix a
// forgotten timer (R11), also one that started on an earlier day.
import { ref } from "vue";
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import { commands } from "../bindings";
import { useTimerStore } from "../stores/timer";
import { call } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { formatElapsed } from "../lib/format";
import { splitLocal } from "../lib/localTime";
import EntryDialog, { type EntryFields } from "./EntryDialog.vue";
import StopAtDialog from "./StopAtDialog.vue";

const timer = useTimerStore();
const { running, elapsed } = storeToRefs(timer);
const attempt = useAttempt();
const editing = ref(false);
const stoppingAt = ref(false);

async function saveEdit(id: number, f: EntryFields) {
  await call(commands.editEntry(id, f.project, f.task, f.start, f.end, f.note));
}
</script>

<template>
  <div class="running-bar" :class="{ active: running }">
    <template v-if="running">
      <span class="dot" aria-hidden="true"></span>
      <span class="label" :title="running.combo.label">{{ running.combo.shortLabel }}</span>
      <span class="elapsed">{{ formatElapsed(elapsed) }}</span>
      <Button label="Edit" severity="secondary" size="small" text @click="editing = true" />
      <Button label="Stop at…" severity="secondary" size="small" text @click="stoppingAt = true" />
      <Button label="Stop" severity="danger" size="small" @click="attempt(() => timer.stop())" />
      <EntryDialog
        v-model:visible="editing"
        header="Edit the running entry"
        :entry="running"
        :day="splitLocal(running.startLocal).date"
        :save="(fields) => saveEdit(running!.id, fields)"
      />
      <StopAtDialog v-model:visible="stoppingAt" :running="running" :save="timer.stop" />
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
