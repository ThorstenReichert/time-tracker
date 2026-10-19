<script setup lang="ts">
// Start or switch from the recents or any combo (R5, R7, R3), and the entries of a chosen day:
// add after the fact, edit, delete with a confirmation (R10, R11), with their notes (R13).
import { onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import Button from "primevue/button";
import { useConfirm } from "primevue/useconfirm";
import { commands, type EntryDto } from "../bindings";
import ComboPicker from "../components/ComboPicker.vue";
import DayBar from "../components/DayBar.vue";
import EntryDialog, { type EntryFields } from "../components/EntryDialog.vue";
import RecentsRow from "../components/RecentsRow.vue";
import { useTimerStore } from "../stores/timer";
import { call, errorMessage } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { taskChoice, type ComboChoice } from "../lib/combo";
import { formatDuration } from "../lib/format";
import { formatSpan } from "../lib/localTime";

const timer = useTimerStore();
const attempt = useAttempt();
const confirm = useConfirm();
const choice = ref<ComboChoice | null>(null);
// Remounts the picker after a start, which clears its inputs.
const pickerKey = ref(0);

async function start() {
  const picked = choice.value;
  if (picked === null) return;
  if (await attempt(() => timer.start(picked.project.project, taskChoice(picked.task)))) {
    choice.value = null;
    pickerKey.value++;
  }
}

const day = ref<string | null>(null);
const entries = ref<EntryDto[] | null>(null);
const loadError = ref<string | null>(null);

async function load() {
  const asked = day.value;
  if (asked === null) return;
  try {
    const result = await call(commands.getDayEntries(asked));
    // A slower answer for a day that's no longer shown is dropped.
    if (asked !== day.value) return;
    entries.value = result;
    loadError.value = null;
  } catch (e) {
    if (asked !== day.value) return;
    loadError.value = errorMessage(e);
  }
}

// Another day's entries are never listed under this day's date, not even when it fails.
watch(day, () => {
  entries.value = null;
  loadError.value = null;
  void load();
});
const unlisten = listen("data-changed", () => void load());
onUnmounted(() => void unlisten.then((stop) => stop()));

/** The running entry counts up each second, from the timer store's clock. */
const duration = (entry: EntryDto) =>
  entry.running && timer.running?.id === entry.id ? timer.elapsed : entry.duration;

// One entry dialog, configured by whatever opened it.
const entryDialog = ref({
  visible: false,
  header: "",
  entry: null as EntryDto | null,
  save: async (_fields: EntryFields) => {},
});

function addEntry() {
  entryDialog.value = {
    visible: true,
    header: "Add an entry",
    entry: null,
    save: async (f) => {
      // The dialog always reads an end for a new entry.
      if (f.end === null) throw new Error("Type the end time as HH:MM.");
      await call(commands.addEntry(f.project, f.task, f.start, f.end, f.note));
    },
  };
}

function editEntry(entry: EntryDto) {
  entryDialog.value = {
    visible: true,
    header: entry.running ? "Edit the running entry" : "Edit the entry",
    entry,
    save: async (f) => {
      await call(commands.editEntry(entry.id, f.project, f.task, f.start, f.end, f.note));
    },
  };
}

function deleteEntry(entry: EntryDto) {
  confirm.require({
    header: "Delete the entry?",
    message: `${entry.combo.label}, ${formatSpan(entry.startLocal, entry.endLocal)}`,
    rejectProps: { label: "Cancel", severity: "secondary", text: true },
    acceptProps: { label: "Delete", severity: "danger" },
    accept: () => void attempt(() => call(commands.deleteEntry(entry.id))),
  });
}
</script>

<template>
  <div class="view">
    <h2>Today</h2>
    <RecentsRow />
    <section>
      <h3>Start a timer</h3>
      <form class="start" @submit.prevent="start">
        <ComboPicker :key="pickerKey" v-model="choice" />
        <Button type="submit" label="Start" :disabled="choice === null" />
      </form>
    </section>

    <section class="entries">
      <div class="entries-head">
        <h3>Entries</h3>
        <Button label="Add entry" severity="secondary" size="small" :disabled="day === null" @click="addEntry" />
      </div>
      <DayBar v-model="day" />
      <p v-if="loadError !== null" class="error">{{ loadError }}</p>
      <p v-if="entries !== null && entries.length === 0" class="muted">No entries on this day.</p>
      <table v-if="entries !== null && entries.length > 0">
        <tbody>
          <tr v-for="entry in entries" :key="entry.id" :data-entry="entry.id">
            <td class="span">
              {{ formatSpan(entry.startLocal, entry.endLocal) }}
              <span v-if="entry.running" class="running">running</span>
            </td>
            <td class="what">
              <div class="label">{{ entry.combo.label }}</div>
              <div v-if="entry.note !== null" class="note muted">{{ entry.note }}</div>
            </td>
            <td class="duration">{{ formatDuration(duration(entry)) }}</td>
            <td class="actions">
              <Button label="Edit" severity="secondary" size="small" text @click="editEntry(entry)" />
              <Button label="Delete" severity="danger" size="small" text @click="deleteEntry(entry)" />
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <EntryDialog
      v-model:visible="entryDialog.visible"
      :header="entryDialog.header"
      :entry="entryDialog.entry"
      :day="day ?? ''"
      :save="entryDialog.save"
    />
  </div>
</template>

<style scoped>
.start {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 0.5rem;
  align-items: start;
}
.entries-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}
.error {
  color: var(--p-red-500);
}
table {
  width: 100%;
  border-collapse: collapse;
}
td {
  padding: 0.3rem 0.5rem;
  border-bottom: 1px solid var(--p-content-border-color);
  vertical-align: top;
}
.span,
.duration {
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}
.span {
  width: 11rem;
}
.duration {
  width: 4rem;
  text-align: right;
}
.note {
  white-space: pre-line;
  font-size: 0.9rem;
}
.actions {
  width: 9rem;
  white-space: nowrap;
  text-align: right;
}
.actions :deep(.p-button) {
  padding-block: 0;
}
.running {
  display: block;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--p-primary-color);
}
.running::before {
  content: "● ";
}
</style>
