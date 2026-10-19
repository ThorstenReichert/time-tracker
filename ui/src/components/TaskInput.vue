<script setup lang="ts">
// Picks a task of a project by typing: matches the project's active tasks and offers
// Create "<text>" when no task has that name (R3). The matching is a tt-core read model.
import { nextTick, ref, watch } from "vue";
import AutoComplete from "primevue/autocomplete";
import { commands } from "../bindings";
import { call } from "../lib/api";
import { taskPicks, type TaskPick } from "../lib/combo";

const model = defineModel<TaskPick | null>({ required: true });
const props = defineProps<{ project: number | null; inputId?: string }>();

// What the input holds: the picked task, or the text typed so far.
const text = ref<TaskPick | string | null>(model.value);
const suggestions = ref<TaskPick[]>([]);
/** The text the suggestions on screen were searched for. */
let suggestionsFor: string | null = null;
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
  suggestionsFor = event.query;
}

// AutoComplete searches 300 ms after the last key. Enter before that picks nothing, and the
// form would go without the task. So Enter on text whose search hasn't answered runs the
// search now, picks its first offer (what Enter picks once the list is there, as in the
// popup) and then submits the form. Only typing on makes it give up: AutoComplete's own
// search may still fire meanwhile. If the search fails, AutoComplete's own search shows the
// list, and Enter picks from it.
async function onKeydown(event: KeyboardEvent) {
  const typed = text.value;
  if (event.key !== "Enter" || props.project === null) return;
  if (typeof typed !== "string" || typed.trim() === "" || typed === suggestionsFor) return;
  event.preventDefault();
  event.stopPropagation();
  const form = (event.target as HTMLInputElement).form;
  let picks: TaskPick[];
  try {
    picks = taskPicks(await call(commands.taskMatches(props.project, typed)));
  } catch {
    return;
  }
  if (text.value !== typed) return;
  const first = picks[0];
  if (first === undefined) return;
  model.value = first;
  text.value = first;
  await nextTick();
  form?.requestSubmit();
}

function select(event: { value: TaskPick }) {
  model.value = event.value;
}
</script>

<template>
  <div @keydown.capture="onKeydown">
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
  </div>
</template>
