<script setup lang="ts">
// Picks a task of a project by typing: matches the project's active tasks and offers
// Create "<text>" when no task has that name (R3). The matching is a tt-core read model.
import { ref, watch } from "vue";
import AutoComplete from "primevue/autocomplete";
import { commands } from "../bindings";
import { call } from "../lib/api";
import { taskPicks, type TaskPick } from "../lib/combo";

const model = defineModel<TaskPick | null>({ required: true });
const props = defineProps<{ project: number | null; inputId?: string }>();

// What the input holds: the picked task, or the text typed so far.
const text = ref<TaskPick | string | null>(model.value);
const suggestions = ref<TaskPick[]>([]);
let latest = 0;

watch(model, (pick) => {
  if (pick !== null || typeof text.value === "object") text.value = pick;
});
watch(
  () => props.project,
  () => {
    if (model.value === null) text.value = null;
  },
);
// Typing drops the pick: only a task chosen from the list counts.
watch(text, (value) => {
  if (typeof value === "string" || value === null) model.value = null;
});

async function complete(event: { query: string }) {
  if (props.project === null) return;
  const request = ++latest;
  const matches = await call(commands.taskMatches(props.project, event.query));
  if (request !== latest) return;
  suggestions.value = taskPicks(matches);
}

function select(event: { value: TaskPick }) {
  model.value = event.value;
}
</script>

<template>
  <AutoComplete
    v-model="text"
    :suggestions="suggestions"
    option-label="name"
    :disabled="project === null"
    :input-id="inputId"
    placeholder="No task (type to find or create)"
    dropdown
    complete-on-focus
    auto-option-focus
    force-selection
    fluid
    @complete="complete"
    @option-select="select"
  >
    <template #option="{ option }">
      <span v-if="option.kind === 'new'">Create "<strong>{{ option.name }}</strong>"</span>
      <span v-else>{{ option.name }}</span>
    </template>
  </AutoComplete>
</template>
