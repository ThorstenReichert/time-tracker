<script setup lang="ts">
// Adds an entry after the fact, or edits one: combo, start, end and note (R10, R13). On the
// running entry, an empty end keeps it running and an end stops it (R11). The core validates
// (R12); a rejection, e.g. "Overlaps Acme / Web 09:00–10:22", is shown and the dialog stays open.
import { computed, ref, useId, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Message from "primevue/message";
import Textarea from "primevue/textarea";
import type { EntryDto, TaskChoiceDto } from "../bindings";
import { errorMessage } from "../lib/api";
import { fromCombo, taskChoice, type ComboChoice } from "../lib/combo";
import { joinLocal, splitLocal } from "../lib/localTime";
import ComboPicker from "./ComboPicker.vue";
import LocalTimeInput from "./LocalTimeInput.vue";

/** What the dialog saves; the times are local ("YYYY-MM-DDTHH:MM:SS"). */
export interface EntryFields {
  project: number;
  task: TaskChoiceDto;
  start: string;
  /** `null` only for the running entry: it keeps running. */
  end: string | null;
  note: string;
}

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  header: string;
  /** The entry to edit; without one, a new entry on `day`. */
  entry?: EntryDto | null;
  /** YYYY-MM-DD, the date a new entry starts with. */
  day: string;
  save: (fields: EntryFields) => Promise<void>;
}>();

const ids = { start: useId(), end: useId(), note: useId() };
const running = computed(() => props.entry?.running ?? false);
// The fields are filled when the dialog opens.
const choice = ref<ComboChoice | null>(null);
const startDate = ref("");
const startTime = ref("");
const endDate = ref("");
const endTime = ref("");
const note = ref("");
const error = ref<string | null>(null);
const busy = ref(false);
// Remounts the picker on open, so it shows the new choice.
const pickerKey = ref(0);

watch(visible, (open) => {
  if (!open) return;
  const entry = props.entry ?? null;
  const start = entry === null ? { date: props.day, time: "" } : splitLocal(entry.startLocal);
  const end = entry?.endLocal ? splitLocal(entry.endLocal) : { date: start.date, time: "" };
  choice.value = entry === null ? null : fromCombo(entry.combo);
  startDate.value = start.date;
  startTime.value = start.time;
  endDate.value = end.date;
  endTime.value = end.time;
  note.value = entry?.note ?? "";
  error.value = null;
  pickerKey.value++;
});

// The end date follows the start date while they're the same day.
watch(
  startDate,
  (now, before) => {
    if (endDate.value === before) endDate.value = now;
  },
  { flush: "sync" },
);

/** The fields to save, or why they can't be read. */
function fields(): EntryFields | string {
  if (choice.value === null) return "Pick a client / project.";
  const start = joinLocal(startDate.value, startTime.value);
  if (start === null) return "Type the start time as HH:MM.";
  let end: string | null = null;
  if (!running.value || endTime.value.trim() !== "") {
    end = joinLocal(endDate.value, endTime.value);
    if (end === null) return "Type the end time as HH:MM.";
  }
  return { project: choice.value.project.project, task: taskChoice(choice.value.task), start, end, note: note.value };
}

async function submit() {
  const read = fields();
  if (typeof read === "string") {
    error.value = read;
    return;
  }
  busy.value = true;
  try {
    await props.save(read);
    visible.value = false;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal :header="header" :style="{ width: '36rem' }">
    <form class="dialog-form" @submit.prevent="submit">
      <span>Client / project / task</span>
      <ComboPicker :key="pickerKey" v-model="choice" />
      <label :for="ids.start">Start</label>
      <LocalTimeInput
        v-model:date="startDate"
        v-model:time="startTime"
        label="Start"
        :time-id="ids.start"
        :autofocus="entry != null"
      />
      <label :for="ids.end">End</label>
      <LocalTimeInput
        v-model:date="endDate"
        v-model:time="endTime"
        label="End"
        :time-id="ids.end"
        :placeholder="running ? 'running' : undefined"
      />
      <small v-if="running" class="muted">Leave the end empty to keep the timer running; an end stops it.</small>
      <label :for="ids.note">Note</label>
      <Textarea :id="ids.note" v-model="note" rows="2" auto-resize fluid />
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button type="submit" label="Save" :loading="busy" />
      </div>
    </form>
  </Dialog>
</template>
