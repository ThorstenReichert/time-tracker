<script setup lang="ts">
// The forgotten stop (R11): stops the running entry at a typed past time instead of now. The
// core rejects a time that isn't after the start or lies in the future (R12).
import { ref, useId, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Message from "primevue/message";
import type { EntryDto } from "../bindings";
import { errorMessage } from "../lib/api";
import { formatClock, joinLocal, splitLocal } from "../lib/localTime";
import LocalTimeInput from "./LocalTimeInput.vue";

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  running: EntryDto;
  /** Stops at this local time ("YYYY-MM-DDTHH:MM:SS"). */
  save: (at: string) => Promise<void>;
}>();

const id = useId();
const date = ref("");
const time = ref("");
const error = ref<string | null>(null);
const busy = ref(false);

// It starts on the start's day: a forgotten timer is usually stopped on the day it started.
watch(visible, (open) => {
  if (!open) return;
  date.value = splitLocal(props.running.startLocal).date;
  time.value = "";
  error.value = null;
});

async function submit() {
  const at = joinLocal(date.value, time.value);
  if (at === null) {
    error.value = "Type the time as HH:MM.";
    return;
  }
  busy.value = true;
  try {
    await props.save(at);
    visible.value = false;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Stop at…" :style="{ width: '28rem' }">
    <form class="dialog-form" @submit.prevent="submit">
      <p class="started">
        <span :title="running.combo.label">{{ running.combo.shortLabel }}</span>, started {{ splitLocal(running.startLocal).date }} {{ formatClock(running.startLocal) }}
      </p>
      <label :for="id">Stopped at</label>
      <LocalTimeInput v-model:date="date" v-model:time="time" label="Stopped at" :time-id="id" autofocus />
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button type="submit" label="Stop" severity="danger" :loading="busy" />
      </div>
    </form>
  </Dialog>
</template>

<style scoped>
.started {
  margin: 0 0 0.5rem;
}
</style>
