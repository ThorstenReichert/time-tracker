<script setup lang="ts">
// Picks a client/project/task combo: the project by fuzzy search over the active combos
// (the quick-switch read model, R6), then the task with R3 behaviour.
import { computed, ref, watch } from "vue";
import AutoComplete from "primevue/autocomplete";
import { commands, type ComboDto } from "../bindings";
import { call } from "../lib/api";
import { fromCombo, type ComboChoice, type TaskPick } from "../lib/combo";
import TaskInput from "./TaskInput.vue";

const model = defineModel<ComboChoice | null>({ required: true });

// What the project input holds: the picked project, or the text typed so far.
const projectText = ref<ComboDto | string | null>(model.value?.project ?? null);
const suggestions = ref<ComboDto[]>([]);
let latest = 0;

watch(model, (choice) => {
  if (choice !== null) projectText.value = choice.project;
  else if (typeof projectText.value === "object") projectText.value = null;
});
// Typing drops the pick: only a combo chosen from the list counts.
watch(projectText, (value) => {
  if (typeof value === "string" || value === null) model.value = null;
});

async function complete(event: { query: string }) {
  const request = ++latest;
  const rows = await call(commands.quickSwitch(event.query));
  if (request === latest) suggestions.value = rows;
}

function select(event: { value: ComboDto }) {
  const choice = fromCombo(event.value);
  projectText.value = choice.project;
  model.value = choice;
}

const task = computed<TaskPick | null>({
  get: () => model.value?.task ?? null,
  set: (pick) => {
    if (model.value !== null) model.value = { ...model.value, task: pick };
  },
});
</script>

<template>
  <div class="combo-picker">
    <AutoComplete
      v-model="projectText"
      :suggestions="suggestions"
      option-label="label"
      placeholder="Client / Project (type to search)"
      dropdown
      complete-on-focus
      auto-option-focus
      fluid
      @complete="complete"
      @option-select="select"
    />
    <TaskInput v-model="task" :project="model?.project.project ?? null" />
  </div>
</template>

<style scoped>
.combo-picker {
  display: grid;
  grid-template-columns: 3fr 2fr;
  gap: 0.5rem;
}
</style>
