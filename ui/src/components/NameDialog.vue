<script setup lang="ts">
// Asks for a name and an optional abbreviation (create or edit a project or task, R1, R24).
// The core validates both; a rejection is shown here and the dialog stays open.
// With `check` (New project, R2c) the typed text is checked while typing: a match with one
// archived project turns the button into "Unarchive project"; a clash blocks the button.
import { computed, ref, useId, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import type { NameCheckDto, ProjectDto } from "../bindings";
import { errorMessage } from "../lib/api";

export interface Naming {
  name: string;
  abbreviation: string | null;
}

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  header: string;
  label: string;
  initial?: Naming;
  /** Gets the abbreviation as typed; the core reads blank text as none (R24). */
  save: (name: string, abbreviation: string | null) => Promise<void>;
  /** The tt-core name check for a new project (R2c). */
  check?: (name: string, abbreviation: string | null) => Promise<NameCheckDto>;
  /** Unarchives the project the check pointed at, as it was (R2a). */
  unarchive?: (project: number) => Promise<void>;
}>();

const ids = { name: useId(), abbreviation: useId() };
const name = ref("");
const abbreviation = ref("");
const error = ref<string | null>(null);
const busy = ref(false);
const checked = ref<NameCheckDto | null>(null);
let latestCheck = 0;

watch(visible, (open) => {
  if (!open) return;
  name.value = props.initial?.name ?? "";
  abbreviation.value = props.initial?.abbreviation ?? "";
  error.value = null;
  checked.value = null;
});

// Only the answer to the newest text counts; the core still checks on save.
watch([name, abbreviation], async ([typedName, typedAbbreviation]) => {
  const check = props.check;
  if (check === undefined || !visible.value) return;
  const request = ++latestCheck;
  let result: NameCheckDto | null;
  try {
    result = await check(typedName, typedAbbreviation);
  } catch {
    result = null; // the save shows the real error
  }
  if (request === latestCheck) checked.value = result;
});

/** `project "Web" (W)`, or `archived project "Web" (W)` (R2d). */
function describe(project: ProjectDto) {
  const abbreviation = project.abbreviation === null ? "" : ` (${project.abbreviation})`;
  return `${project.archived ? "archived " : ""}project "${project.name}"${abbreviation}`;
}

const toUnarchive = computed(() =>
  checked.value?.kind === "unarchive" && props.unarchive !== undefined ? (checked.value.projects[0] ?? null) : null,
);
const taken = computed(() => (checked.value?.kind === "taken" ? checked.value.projects : []));

async function submit() {
  if (taken.value.length > 0) return;
  busy.value = true;
  try {
    const project = toUnarchive.value;
    if (project !== null) await props.unarchive!(project.id);
    else await props.save(name.value, abbreviation.value);
    visible.value = false;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal :header="header" :style="{ width: '28rem' }">
    <form class="dialog-form" @submit.prevent="submit">
      <label :for="ids.name">{{ label }}</label>
      <InputText :id="ids.name" v-model="name" autofocus fluid />
      <label :for="ids.abbreviation">Abbreviation (optional)</label>
      <InputText :id="ids.abbreviation" v-model="abbreviation" fluid />
      <Message v-if="toUnarchive" severity="info" class="name-check">
        {{ `The ${describe(toUnarchive)} exists. Unarchive it to use it again, or choose another name.` }}
      </Message>
      <Message v-else-if="taken.length > 0" severity="error" class="name-check">
        Already used by {{ taken.map(describe).join(", ") }}
      </Message>
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button
          type="submit"
          :label="toUnarchive ? 'Unarchive project' : 'Save'"
          :loading="busy"
          :disabled="taken.length > 0"
        />
      </div>
    </form>
  </Dialog>
</template>
